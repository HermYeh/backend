
use log::info;
use sqlite::{self, Row};
use std::net::{Shutdown,TcpListener, TcpStream};
use std::{
    io::{ErrorKind, Read, Write},
  
};
use warp::Filter;

use std::convert::Infallible;
use env_logger;
#[macro_use]
use log;

use std::mem::size_of;

use serde::{Deserialize, Serialize};
fn handle_connection( vector:Vec<(String,String,bool)>) {
       
       
        let connection = sqlite::Connection::open("order.db").unwrap();
        let mut query = "CREATE TABLE IF NOT EXISTS data (order_number TEXT PRIMARY KEY, check_in TEXT);";
        println!("query{}",query);
        connection.execute(query).unwrap();
        
        query = "DELETE FROM data;";
        connection.execute(query).unwrap();
        for each in vector.iter(){
            let query="INSERT OR IGNORE INTO data (order_number, check_in) VALUES ('".to_owned()+&each.0+"','"+&each.1+"');
            UPDATE data SET check_in ='"+&each.1+"' WHERE order_number='"+&each.0+"';";
            
            println!("query{}",query);
            connection.execute(&query).unwrap();
        }
       
        
        
    }
    
  
    
    const LOCAL: &str = "127.0.0.1:6000";
    const MSG_SIZE: usize = 32;
    #[derive(Serialize, Deserialize)]
struct Message {
    vector:Vec<(String,String,bool)>,
}
#[derive(Debug, Copy, Clone)]
#[repr(C, align(8))]
struct FileHeader {
    size: u32,
    
}
use pretty_env_logger;
#[derive(Serialize, Deserialize)]
struct Orders{
order_number:String,
checkin: String,

}

use std::slice;
const BUF_LEN: usize = 4096;
fn handle_client(mut stream: TcpStream)->  std::io::Result<()> {
    println!("incoming connection from: {}", stream.peer_addr()?);
    let mut buf_file_header = [0; size_of::<FileHeader>()];
    println!("buf_file_header{:?}",buf_file_header);
    stream.read_exact(&mut buf_file_header)?;
    let mut total_orders:Vec<(String,String,bool)>=Vec::new();
    let file_header: FileHeader = unsafe { *(buf_file_header.as_ptr() as *const _) };
    let file_size = file_header.size as usize;
    let mut buf = [0; BUF_LEN];
    println!("file_header.size{:?}",file_header.size);
       let conn =sqlite::Connection::open("order.db").unwrap();
       let query = "SELECT * FROM data";
    let mut statement=conn.prepare(query).unwrap();
     while let sqlite::State::Row= statement.next().unwrap(){
     
     let orderid= statement.read(0).unwrap();
     let checkin=statement.read(1).unwrap();
     total_orders.push((orderid,checkin,false));
     } ;
    if file_size==0{
 
     let message = Message { vector:total_orders};
    
     let data = serde_json::to_vec(&message).unwrap();
        
          
      unsafe{
      stream.write(
          slice::from_raw_parts(
              &FileHeader {
                  size: data.len() as u32,
              } as *const _ as *const u8,
              size_of::<FileHeader>(),
          )
      ).unwrap();
      }        
      stream.write(&data).unwrap();
    }else{
    let mut readen_size = 0;
        while readen_size < file_size {
            let read_size: usize = if file_size - readen_size < BUF_LEN {
                println!("readen_size.size{:?}",readen_size);
                file_size - readen_size
            
            } else {
                println!("readen_size.size{:?}",readen_size);
                BUF_LEN
            };
            stream.read_exact(&mut buf[0..read_size])?;
            readen_size += read_size;
            let message: Message = serde_json::from_slice(&buf[0..read_size]).unwrap();
            println!("Received vector: {:?}", message.vector);
            handle_connection(message.vector);
        }
    }

    Ok(())
}


use std::collections::HashMap;
use warp::{
    http::{Response, StatusCode},
};

#[derive(Deserialize, Serialize)]
struct MyObject {
    key1: String,
    key2: u32,
}

#[tokio::main]
async fn main() {
    // POST /data
    let data_route = warp::post()
        .and(warp::path("data"))
        .and(warp::body::json())
        .and_then(handle_post);
    
    let query = warp::get()
        .and(warp::path("load"))
       .and(warp::body::json())
        .and_then(load);
        
        warp::serve(data_route.or(query))
        .run(([127, 0, 0, 1], 3030))
        .await;
    
}
async fn load(my_data: Message) -> Result<impl warp::Reply, Infallible> {

    println!("Received data: {:?}", my_data.vector);
    
    let mut total_orders=Vec::new();
    let conn =sqlite::Connection::open("order.db").unwrap();
    let query = "SELECT * FROM data";
 let mut statement=conn.prepare(query).unwrap();
  while let sqlite::State::Row= statement.next().unwrap(){
  
  let orderid= statement.read(0).unwrap();
  let checkin=statement.read(1).unwrap();
  total_orders.push((orderid,checkin,false));
  } ;

  
  let message = Message { vector:total_orders};
 

     
    Ok(warp::reply::json(&message))
}
async fn handle_post(my_data: Message) -> Result<impl warp::Reply, Infallible> {
    println!("Received data: {:?}", my_data.vector);
    handle_connection(my_data.vector.clone());
    Ok(warp::reply::json(&my_data))
}