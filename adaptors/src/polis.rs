use std::collections::HashMap;

use sea_query::{
    enum_def, ColumnDef, Expr, Iden, PostgresQueryBuilder, Query, SelectStatement, Table,
};
use sea_query_binder::SqlxBinder;
use serde::Deserialize;
use sqlx::{types::BigDecimal, FromRow};
use sqlx_postgres::{PgPool, PgPoolOptions};
use thiserror::Error;

#[derive(Debug, Deserialize, FromRow)]
#[enum_def(table_name = "users")]
pub struct User {
    pub uid: i32,
    pub username: Option<String>,
    pub email: Option<String>,
    pub site_owner: bool,
}

#[derive(Debug, Deserialize, FromRow)]
#[enum_def(table_name = "participants")]
pub struct Participants {
    pub pid: i32,
    pub uid: i32,
    pub zid: i32,
}

#[derive(Debug, Deserialize, FromRow)]
#[enum_def(table_name = "xids")]
pub struct Xid {
    pub uid: i32,
    pub xid: String,
}

#[derive(Debug, Deserialize, FromRow)]
#[enum_def(table_name = "votes")]
pub struct Vote {
    pub pid: i32,
    pub zid: i32,
    pub vote: i16,
    pub tid: i32,
    pub high_priority: bool,
}

#[derive(Debug, Deserialize)]
pub struct VoteSummary {
    A: i32,
    D: i32,
    S: i32,
}

#[derive(Debug, Deserialize)]
pub struct GroupVotes {
    votes: HashMap<String, VoteSummary>,
    #[serde(rename = "n-members")]
    n_members: i32,
}

#[derive(Debug, Deserialize)]
pub struct GroupCluster {
    id: i32,
    center: [f32; 2],
    members: Vec<i32>,
}
#[derive(Debug, Deserialize)]
pub struct SubGroupCluster {
    id: i32,
    center: [f32; 2],
    members: Vec<i32>,
    #[serde(rename = "parent-id")]
    parent_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct Pca {
    comps: Vec<Vec<f32>>,
    center: Vec<f32>,
    #[serde(rename = "comment-extremity")]
    comment_extremity: Vec<f32>,
    #[serde(rename = "comment-projection")]
    comment_projection: Vec<Vec<f32>>,
}

#[derive(Debug, Deserialize)]
pub struct MathData {
    n: i32,
    pca: Pca,
    zid: i32,
    tids: Vec<i32>,
    #[serde(rename = "mod-in")]
    mod_in: Vec<i32>,
    #[serde(rename = "n-cmts")]
    n_cmts: i32,
    #[serde(rename = "in-conv")]
    in_conv: Vec<i32>,
    #[serde(rename = "mod-out")]
    mod_out: Vec<i32>,
    #[serde(rename = "group-votes")]
    group_votes: HashMap<String, GroupVotes>,
    #[serde(rename = "group-clusters")]
    group_clusters: Vec<GroupCluster>,
    #[serde(rename = "user-vote-counts")]
    user_vote_counts: HashMap<String, i32>,
    #[serde(rename = "subgroup-clusters")]
    subgroup_clusters: HashMap<String, Vec<SubGroupCluster>>,
    #[serde(rename = "comment-priorities")]
    comment_priorities: HashMap<String, f32>,
    #[serde(rename = "group-aware-consensus")]
    group_aware_consensus: HashMap<String, f32>,
}

#[derive(Debug, Deserialize, FromRow)]
#[enum_def(table_name = "math_main")]
pub struct Math {
    zid: i32,
    data: sqlx::types::Json<MathData>,
}

#[derive(Debug, Deserialize, FromRow)]
#[enum_def(table_name = "comments")]
pub struct Comment {
    pub tid: i32,
    pub pid: i32,
    pub zid: i32,
    pub uid: i32,
    pub txt: String,
    pub lang: Option<String>,
    pub anon: bool,
}

pub struct PolisConnector {
    db: PgPool,
    server_url: String,
}

#[derive(Error, Debug)]
pub enum PolisConnectionError {
    #[error("data store disconnected")]
    FailedToConnect(#[from] sqlx::Error),
}

impl PolisConnector {
    pub async fn new(
        db_connection_str: &str,
        server_url: &str,
    ) -> Result<Self, PolisConnectionError> {
        let db = PgPoolOptions::new()
            .max_connections(5)
            .connect(db_connection_str)
            .await?;

        return Ok(Self {
            db,
            server_url: server_url.into(),
        });
    }

    pub async fn get_active_users(&self, zid: i32) -> Result<Vec<User>, PolisConnectionError> {
        let (sql, values) = Query::select()
            .columns([
                (UserIden::Table, UserIden::Uid),
                (UserIden::Table, UserIden::Username),
                (UserIden::Table, UserIden::Email),
                (UserIden::Table, UserIden::SiteOwner),
            ])
            .from(UserIden::Table)
            .build_sqlx(PostgresQueryBuilder);

        println!("users {sql} ");
        let users = sqlx::query_as_with::<_, User, _>(&sql, values)
            .fetch_all(&self.db)
            .await?;

        Ok(users)
    }

    pub async fn get_votes(&self, zid: i32) -> Result<Vec<Vote>, PolisConnectionError> {
        let (sql, values) = Query::select()
            .columns([
                VoteIden::Pid,
                VoteIden::Zid,
                VoteIden::Tid,
                VoteIden::Vote,
                VoteIden::HighPriority,
            ])
            .from(VoteIden::Table)
            .and_where(Expr::col(VoteIden::Zid).eq(zid))
            .build_sqlx(PostgresQueryBuilder);

        println!("SQL: {}", sql);
        let votes = sqlx::query_as_with::<_, Vote, _>(&sql, values)
            .fetch_all(&self.db)
            .await?;

        Ok(votes)
    }

    pub async fn get_comments(&self, zid: i32) -> Result<Vec<Comment>, PolisConnectionError> {
        let (sql, values) = Query::select()
            .columns([
                CommentIden::Tid,
                CommentIden::Pid,
                CommentIden::Zid,
                CommentIden::Uid,
                CommentIden::Txt,
                CommentIden::Lang,
                CommentIden::Anon,
            ])
            .from(CommentIden::Table)
            .and_where(Expr::col(CommentIden::Zid).eq(zid))
            .build_sqlx(PostgresQueryBuilder);

        println!("SQL: {}", sql);
        let comments = sqlx::query_as_with::<_, Comment, _>(&sql, values)
            .fetch_all(&self.db)
            .await?;

        Ok(comments)
    }

    pub async fn get_participants(
        &self,
        zid: i32,
    ) -> Result<Vec<Participants>, PolisConnectionError> {
        let (sql, values) = Query::select()
            .columns([
                (ParticipantsIden::Table, ParticipantsIden::Pid),
                (ParticipantsIden::Table, ParticipantsIden::Uid),
                (ParticipantsIden::Table, ParticipantsIden::Zid),
            ])
            .from(ParticipantsIden::Table)
            .and_where(Expr::col(ParticipantsIden::Zid).eq(zid))
            .build_sqlx(PostgresQueryBuilder);

        let participants = sqlx::query_as_with::<_, Participants, _>(&sql, values)
            .fetch_all(&self.db)
            .await?;

        Ok(participants)
    }

    pub async fn get_xids(&self) -> Result<Vec<Xid>, PolisConnectionError> {
        let (sql, values) = Query::select()
            .columns([XidIden::Uid, XidIden::Xid])
            .from(XidIden::Table)
            .build_sqlx(PostgresQueryBuilder);

        println!("SQL: {}", sql);
        let xid = sqlx::query_as_with::<_, Xid, _>(&sql, values)
            .fetch_all(&self.db)
            .await?;

        Ok(xid)
    }

    pub async fn get_math(&self, zid: i32) -> Result<Math, PolisConnectionError> {
        let (sql, values) = Query::select()
            .columns([MathIden::Zid, MathIden::Data])
            .from(MathIden::Table)
            .and_where(Expr::col(MathIden::Zid).eq(zid))
            .build_sqlx(PostgresQueryBuilder);

        println!("SQL: {}", sql);
        let math = sqlx::query_as_with::<_, Math, _>(&sql, values)
            .fetch_one(&self.db)
            .await?;

        Ok(math)
    }

    pub async fn get_lookups(
        &self,
        zid: i32,
    ) -> Result<
        (
            HashMap<i32, i32>,
            HashMap<i32, String>,
            HashMap<i32, String>,
        ),
        PolisConnectionError,
    > {
        let xids = self.get_xids().await?;
        let participants = self.get_participants(zid).await?;

        // Generate uid to xid lookup
        let uids: Vec<i32> = xids.iter().map(|p| p.uid).collect();
        let xids: Vec<String> = xids.iter().map(|p| p.xid.clone()).collect();

        let uids_to_xids: HashMap<i32, String> = uids.into_iter().zip(xids.into_iter()).collect();

        // Generate pid to uids

        let uids: Vec<i32> = participants.iter().map(|p| p.uid).collect();
        let pids: Vec<i32> = participants.iter().map(|p| p.pid).collect();

        let pids_to_uids: HashMap<i32, i32> = pids.into_iter().zip(uids.into_iter()).collect();

        // Generate pids to xids

        let mut pids_to_xids: HashMap<i32, String> = HashMap::new();

        for (pid, uid) in &pids_to_uids {
            if let Some(xid) = uids_to_xids.get(uid) {
                pids_to_xids.insert(*pid, xid.clone());
            } else {
                println!("non matching uid {uid}");
            }
        }

        Ok((pids_to_uids, uids_to_xids, pids_to_xids))
    }

    pub async fn get_group_membership(
        &self,
        zid: i32,
    ) -> Result<HashMap<String, i32>, PolisConnectionError> {
        let (_, _, user_lookup) = self.get_lookups(zid).await?;
        let mut group_membership = HashMap::new();
        let math = self.get_math(zid).await?;
        let clusters = &math.data.group_clusters;
        for cluster in clusters {
            let cluster_id = cluster.id;
            for member in &cluster.members {
                if let Some(xid) = user_lookup.get(member) {
                    group_membership.insert(xid.clone(), cluster_id);
                } else {
                    println!("miss {member}");
                }
            }
        }

        Ok(group_membership)
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use super::*;

    #[tokio::test]
    async fn getting_votes() {
        let polis = PolisConnector::new(
            "postgres://postgres:polis@localhost:5431/polis-dev",
            "https://localhost:3000",
        )
        .await
        .unwrap();

        let users = polis.get_active_users(12).await.unwrap();
        println!("{users:#?}");
    }

    #[tokio::test]
    async fn getting_comments() {
        let polis = PolisConnector::new(
            "postgres://postgres:polis@localhost:5431/polis-dev",
            "https://localhost:3000",
        )
        .await
        .unwrap();

        let comments = polis.get_comments(12).await.unwrap();
        println!("{comments:#?}");
    }

    #[tokio::test]
    async fn test_deserialize_math() {
        let math_string =
            std::fs::read_to_string("/home/stuart/crown_shy/comhairle/adaptors/data_example.json")
                .unwrap();
        let math: MathData = serde_json::from_str(&math_string).unwrap();
        println!("{math:#?}");
    }

    #[tokio::test]
    async fn getting_math() {
        let polis = PolisConnector::new(
            "postgres://postgres:polis@localhost:5431/polis-dev",
            "https://localhost:3000",
        )
        .await
        .unwrap();

        let math = polis.get_math(12).await.unwrap();
        println!("{math:#?}");
    }
}
