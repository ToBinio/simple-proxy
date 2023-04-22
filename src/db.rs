use std::env;
use std::sync::mpsc;

use diesel::{Connection, ExpressionMethods, MysqlConnection, QueryDsl, RunQueryDsl};
use tokio::sync::{broadcast, oneshot};
use tracing::{debug, info};

use crate::models::Tunnel;
use crate::schema::tunnels::dsl::tunnels;
use crate::schema::tunnels::id;

pub struct DB {}

impl DB {
    pub fn start() -> mpsc::Sender<DBMessage> {
        let (sender, receiver) = mpsc::channel();

        tokio::spawn(async move {
            let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
            let mut db_connection = MysqlConnection::establish(&database_url)
                .expect(&*format!("Error connecting to {}", database_url));
            info!("Connected to DB");

            let (update_sender, update_receiver) = broadcast::channel(8);

            info!("DB-Service listening");
            while let Ok(msg) = receiver.recv() {
                match msg {
                    DBMessage::Subscribe(sender) => {
                        sender
                            .send(update_receiver.resubscribe())
                            .expect("TODO: panic message");

                        update_sender.send(()).unwrap();
                    }
                    DBMessage::GetALl(sender) => {
                        let vec = tunnels
                            .load::<Tunnel>(&mut db_connection)
                            .expect("Error loading connections");

                        sender.send(vec).unwrap();
                    }
                    DBMessage::Remove(search_id) => {
                        info!("Deleting tunnel with id {}", search_id);

                        diesel::delete(tunnels.filter(id.eq(search_id)))
                            .execute(&mut db_connection)
                            .expect("Error deleting tunnel");

                        update_sender.send(()).unwrap();
                        debug!("send update request");
                    }
                }
            }
        });

        sender
    }
}

#[derive(Debug)]
pub enum DBMessage {
    Subscribe(oneshot::Sender<broadcast::Receiver<()>>),
    GetALl(oneshot::Sender<Vec<Tunnel>>),
    Remove(i32),
}
