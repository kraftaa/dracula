use super::prelude::*;
use diesel::debug_query;
use diesel::query_builder::QueryFragment;
use diesel::{
    dsl::{Limit, Offset},
    prelude::*,
    query_dsl::{
        methods::{LimitDsl, OffsetDsl},
        LoadQuery,
    },
    result::Error,
};
use std::{collections::VecDeque, marker::PhantomData};

#[derive(Debug)]
pub struct RecordIter<'conn, Record, Query, Conn> {
    conn: &'conn Conn,
    query: Query,
    /// The index of the next record to fetch from the server
    cursor: i64,
    _buffer: VecDeque<Record>,
    record_type: PhantomData<Record>,
}

impl<'conn, Record, Query, Conn> Iterator for RecordIter<'conn, Record, Query, Conn>
where
    Query: OffsetDsl + Clone,
    Offset<Query>: LimitDsl,
    Limit<Offset<Query>>: LoadQuery<Conn, Record> + QueryFragment<diesel::pg::Pg>,
{
    type Item = Result<Vec<Record>, Error>;

    // &mut self -> Option<Result<Vec<Record>, Error>>
    fn next(&mut self) -> Option<Self::Item> {
        trace!("on page {}", self.cursor);

        let fetch_amt = 100;
        let query = self
            .query
            .clone()
            .offset(self.cursor * fetch_amt)
            .limit(fetch_amt);

        trace!("sql: {}", debug_query(&query));

        self.cursor += 1;

        let res = query.load(self.conn);

        match res {
            Err(err) => {
                error!("query result err: {:?}", err);
                Some(Err(err))
            }
            Ok(res) => {
                if res.is_empty() {
                    None
                } else {
                    Some(Ok(res))
                }
            }
        }
    }
}

pub struct PageIter<'conn, Record, Query, Conn> {
    conn: &'conn Conn,
    query: Query,
    /// The index of the next record to fetch from the server
    cursor: i64,
    limit: i64,
    phantom: PhantomData<Record>,
}

impl<'conn, Record, Query, Conn> PageIter<'conn, Record, Query, Conn> {
    pub fn new(conn: &'conn Conn, query: Query, limit: i64) -> Self {
        PageIter {
            conn,
            query,
            limit,
            cursor: 0,
            phantom: PhantomData,
        }
    }
    pub fn f() -> Result<(), Error> {
        Ok(())
    }
}

impl<'conn, Record, Query, Conn> Iterator for PageIter<'conn, Record, Query, Conn>
where
    Query: OffsetDsl + Clone,
    Offset<Query>: LimitDsl,
    Limit<Offset<Query>>: LoadQuery<Conn, Record> + QueryFragment<diesel::pg::Pg>,
{
    type Item = Result<Vec<Record>, Error>;

    // &mut self -> Option<Result<Vec<Record>, Error>>
    fn next(&mut self) -> Option<Self::Item> {
        trace!("on page {}", self.cursor);

        let query = self
            .query
            .clone()
            .offset(self.cursor * self.limit)
            .limit(self.limit);

        trace!("sql: {}", debug_query(&query));

        self.cursor += 1;

        let res = query.load(self.conn);

        match res {
            Err(err) => {
                error!("query result err: {:?}", err);
                Some(Err(err))
            }
            Ok(res) => {
                if res.is_empty() {
                    None
                } else {
                    Some(Ok(res))
                }
            }
        }
    }
}
