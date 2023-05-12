use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::net::UdpSocket;
use tokio::time::{self, Duration};
// use tokio::sync::mpsc::Sender;

#[derive(Debug, Serialize, Deserialize)]
struct Player {
    address: String,
    id: String,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    timestamp: f32,
}

impl Player {
    fn set_address(&mut self, address: String) {
        self.address = address;
    }
}

#[derive(Debug)]
struct Room {
    id: String,
    players: HashMap<String, Player>,
}

impl Room {
    fn new(id: String) -> Self {
        Self {
            id,
            players: HashMap::new(),
        }
    }

    fn add_player(&mut self, player: Player) {
        if self.players.contains_key(&player.id) {
            self.update_player(player);
            return
        };
        self.players.insert(player.id.to_owned(), player);
    }

    fn update_player(&mut self, player: Player) {
        if let Some(existing_player) = self.players.get_mut(&player.id) {
            existing_player.x = player.x;
            existing_player.y = player.y;
            existing_player.vx = player.vx;
            existing_player.vy = player.vy;
            existing_player.timestamp = player.timestamp;
        }
    }
}

async fn handle_message(buf: &[u8], rooms: &mut Vec<Room>, remote: SocketAddr,) -> Result<Player, Box<dyn std::error::Error>> {
    let mut player: Player = match serde_json::from_slice(buf) {
        Ok(player) => player,
        Err(err) => {
            println!("error parsing json: {:?}", err);
            return Err(err.into())
        },
    };
    player.set_address(remote.to_string());

    println!("player: {:?}", player);
    Ok(player)
}

fn update_room_state(room: &mut Room) {
    for player in room.players.values_mut() {
        player.x += player.vx;
        player.y += player.vy;
    }
}

async fn send_room_state(socket: &UdpSocket, room: &Room) -> Result<(), Box<dyn std::error::Error>> {
    for player in room.players.values() {
        let message = format!("Player {} is now at ({}, {})", player.id, player.x, player.y);
        println!("{}", message);

        socket.send_to(message.as_bytes(), &player.address).await?;
    }

    Ok(())
}

async fn run_server(addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    let socket = UdpSocket::bind(addr).await?;
    let mut buf = vec![0u8; 1024];
    let mut rooms = Vec::new();
    let mut interval = time::interval(Duration::from_millis(50)); // 20 ticks per second

    loop {
        interval.tick().await;

        while let Ok((n, remote)) = socket.recv_from(&mut buf).await {
            if let Ok(player) = handle_message(&buf[..n], &mut rooms, remote).await {
                let room = match rooms.iter_mut().find(|r| r.players.len() <= 5) {
                    Some(room) => room,
                    None => {
                        let room_id = player.id.to_owned();
                        let mut new_room = Room::new(room_id);
                        new_room.add_player(player);
                        rooms.push(new_room);
                        continue;
                    }
                };
                room.add_player(player);
            }
        }

        for room in &mut rooms {
            update_room_state(room);
            send_room_state(&socket, room).await?;
        }
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:3000";
    println!("Listening on {}", addr);

    run_server(addr).await?;

    Ok(())
}