use crate::{db, models::Subscriber, Result};

pub use models::*;

mod models;
mod query;

/// ユーザーをデータベースに登録する。
/// ユーザーがいなくて登録できなかった場合は `false` を返す。
pub async fn register_user(username: impl AsRef<str>, user_id: u64, guild_id: u64) -> Result<bool> {
    let res = query::with_after(username.as_ref(), Some(1), None).await?;

    let user = match res {
        Response::Data(data) => data.user,
        Response::Errors(e) => return Err(format!("{:?}", e).into()),
    };

    let Some(user) = user else {
        return Ok(false);
    };

    let mut conn = db::connect()?;
    db::insert_or_update_subscriber(
        &mut conn,
        user_id,
        guild_id,
        username,
        user.activities.page_info.end_cursor.as_deref(),
        user.activities
            .edges
            .first()
            .map(|edge| edge.item.created_at()),
    )?;
    Ok(true)
}

pub async fn get_new_activities(subscriber: &Subscriber) -> Result<Vec<ActivityItem>> {
    let activity_connection = match query::with_after(
        &subscriber.annict_name,
        None,
        subscriber.end_cursor.as_deref(),
    )
    .await?
    {
        Response::Data(data) => {
            let Some(user) = data.user else {
                return Err(format!("no such user: {}", subscriber.annict_name).into());
            };
            user.activities
        }
        Response::Errors(e) => return Err(format!("{:?}", e).into()),
    };
    let after_activities: Vec<_> = activity_connection
        .edges
        .into_iter()
        .map(|edge| edge.item)
        .collect();

    let mut conn = db::connect()?;

    // 元々 end_cursor が None の場合は現時点ですべて取得し終えているので、特に何もする必要はない
    if subscriber.end_cursor.is_none() {
        db::update_subscriber_info(
            &mut conn,
            subscriber.id,
            activity_connection.page_info.end_cursor.as_deref(),
            after_activities.first().map(|act| act.created_at()),
        )?;
        return Ok(after_activities);
    }

    // end_cursor が Some だった場合、データの削除等が起こったときに取得できていない
    // アクティビティがある可能性があるので、過去も振り返って確認する
    // 既に見たアクティビティあるので、last_activity_date は必ず Some => unwrap は必ず成功
    let last_activity_date = db::get_last_activity_date(&mut conn, subscriber.id)?.unwrap();
    let mut end_cursor = activity_connection.page_info.end_cursor;
    let mut reversed_before_activities = vec![];
    let mut cursor = activity_connection.page_info.start_cursor;
    loop {
        let res =
            query::query_with_before(&subscriber.annict_name, Some(1), cursor.as_deref()).await?;
        let user = match res {
            Response::Data(data) => data.user,
            Response::Errors(e) => return Err(format!("{:?}", e).into()),
        };
        let Some(user) = user else {
            return Err(format!("no such user {}", subscriber.annict_name).into());
        };

        let Some(edge) = user.activities.edges.into_iter().next() else {
            // これ以上過去にアクティビティはないので終了
            break;
        };

        cursor = Some(edge.cursor);
        if end_cursor.is_none() {
            // end_cursor が None の場合はこれが一番新しいアクティビティなのでセット
            end_cursor = cursor.clone();
        }
        if edge.item.created_at() <= last_activity_date {
            // 過去に最後まで見たアクティビティよりも過去のデータなら、これより前は見ている
            break;
        }
        reversed_before_activities.push(edge.item);
    }
    let activities: Vec<_> = reversed_before_activities
        .into_iter()
        .rev()
        .chain(after_activities.into_iter())
        .collect();

    db::update_subscriber_info(
        &mut conn,
        subscriber.id,
        end_cursor.as_deref(),
        activities.last().map(|act| act.created_at()),
    )?;

    Ok(activities)
}
