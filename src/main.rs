use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::net::UdpSocket;
// use tokio::sync::mpsc::Sender;

#[derive(Debug, Serialize, Deserialize)]
struct Player {
    // address: String,
    id: String,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    pub timestamp: f32,
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

async fn handle_message(socket: &UdpSocket, remote: SocketAddr, buf: &[u8], rooms: &mut Vec<Room>) -> Result<(), Box<dyn std::error::Error>> {
    // println!("here rooms: {:?}", rooms.iter().map(|room| room.id.to_owned()).collect::<Vec<String>>());
    let player: Player = match serde_json::from_slice(buf) {
        Ok(player) => player,
        Err(err) => {
            println!("error parsing json: {:?}", err);
            return Ok(())
        },
    };
    println!("player: {:?}", player);
    let room = match rooms.iter_mut().find(|r| r.players.len() <= 5) {
        Some(room) => room,
        None => {
            let room_id = player.id.to_owned();
            let mut new_room = Room::new(room_id);
            new_room.add_player(player);
            rooms.push(new_room);
            return Ok(());
        }
    };

    room.add_player(player);

    let response = serde_json::to_string(&room.players)?;
    // println!("Sending response to {}: {}", remote, response);
    socket.send_to(response.as_bytes(), &remote).await?;
    
    Ok(())
}


async fn run_server(addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    let socket = UdpSocket::bind(addr).await?;
    let mut buf = vec![0u8; 1024];
    let mut rooms = Vec::new();
    println!("anything");
    loop {
        let (n, remote) = socket.recv_from(&mut buf).await?;
        if let Err(e) = handle_message(&socket, remote, &buf[..n], &mut rooms).await {
            println!("Error handling message: {}", e);
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