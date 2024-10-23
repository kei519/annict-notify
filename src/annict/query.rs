use std::sync::{
    atomic::{AtomicUsize, Ordering},
    LazyLock,
};

use reqwest::Client;
use serde::Serialize;

use crate::{annict::models::UserQuery, get_env, Result};

use super::models::{Response, UserWithActivities};

static CLIENT: LazyLock<Client> = LazyLock::new(Client::new);

pub(super) async fn with_after(
    username: &str,
    last: Option<i32>,
    after: Option<&str>,
) -> Result<Response<UserQuery<UserWithActivities>>> {
    let query = Query {
        query: r#"query User ($name: String!, $last: Int, $after: String) {
                user(username: $name) { ...userFrag }
            }

            fragment userFrag on User {
                username
                name
                activities(last: $last, after: $after) {
                    edges { item { ...activityFrag } cursor }
                    pageInfo { endCursor }
                }
                avatarUrl
            }

            fragment recordFrag on Record {
                work { ...workFrag }
                createdAt
                comment
                episode { number numberText title }
                rating
                ratingState
            }

            fragment activityFrag on ActivityItem {
                __typename
                ... on MultipleRecord {
                    createdAt
                    records { edges { node { ...recordFrag } } }
                    work { ...workFrag }
                }
                ... on Record { ...recordFrag }
                ... on Review {
                    work { ...workFrag }
                    body
                    createdAt
                    ratingAnimationState
                    ratingCharacterState
                    ratingMusicState
                    ratingOverallState
                    ratingStoryState
                }
                ... on Status { work { ...workFrag } createdAt state }
            }

            fragment workFrag on Work {
                title
            }
        "#,
        variables: UserVariableForAfter {
            name: username,
            last,
            after,
        },
    };

    let res = post_query(query).await?;
    Ok(serde_json::from_str(&res)?)
}

pub(super) async fn query_with_before(
    username: &str,
    last: Option<i32>,
    before: Option<&str>,
) -> Result<Response<UserQuery<UserWithActivities>>> {
    let query = Query {
        query: r#"query User ($name: String!, $last: Int, $before: String) {
                user(username: $name) { ...userFrag }
            }

            fragment userFrag on User {
                username
                name
                activities(last: $last, before: $before) {
                    edges { item { ...activityFrag } cursor }
                    pageInfo { endCursor }
                }
                avatarUrl
            }

            fragment recordFrag on Record {
                work { ...workFrag }
                createdAt
                comment
                episode { number numberText title }
                rating
                ratingState
            }

            fragment activityFrag on ActivityItem {
                __typename
                ... on MultipleRecord {
                    createdAt
                    records { edges { node { ...recordFrag } } }
                    work { ...workFrag }
                }
                ... on Record { ...recordFrag }
                ... on Review {
                    work { ...workFrag }
                    body
                    createdAt
                    ratingAnimationState
                    ratingCharacterState
                    ratingMusicState
                    ratingOverallState
                    ratingStoryState
                }
                ... on Status { work { ...workFrag } createdAt state }
            }

            fragment workFrag on Work {
                title
            }
        "#,
        variables: UserVariableForBefore {
            name: username,
            last,
            before,
        },
    };

    let res = post_query(query).await?;
    Ok(serde_json::from_str(&res)?)
}

async fn post_query<Q: AsRef<str> + Serialize, V: Serialize>(query: Query<Q, V>) -> Result<String> {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    let count = COUNTER.fetch_add(1, Ordering::Relaxed);

    let request = CLIENT
        .post("https://api.annict.com/graphql")
        .bearer_auth(get_env("ANNICT_TOKEN")?)
        .header("Content-Type", "application/json")
        .json(&query);
    tracing::trace!("request({}) = {:?}", count, request,);

    let response = request.send().await?.text().await?;
    tracing::trace!("response({}) = {:?}", count, response);

    Ok(response)
}

#[derive(Debug, Serialize)]
struct Query<Q: AsRef<str> + Serialize, V: Serialize> {
    query: Q,
    variables: V,
}

#[derive(Debug, Serialize)]
struct UserVariableForAfter<S: AsRef<str>, T: AsRef<str>> {
    name: S,
    last: Option<i32>,
    after: Option<T>,
}

#[derive(Debug, Serialize)]
struct UserVariableForBefore<S: AsRef<str>, T: AsRef<str>> {
    name: S,
    last: Option<i32>,
    before: Option<T>,
}
