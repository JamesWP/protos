use std::{
    io::{Read, Write},
};

fn handle_thread(mut stream: std::net::TcpStream) -> std::io::Result<()> {
    let mut buf = [0u8; 9];
    let mut prices = std::collections::BTreeMap::<i32, i32>::new();
    loop {
        stream.read_exact(&mut buf)?;
        let t = buf[0] as char;
        let i1 = i32::from_be_bytes(buf[1..=4].try_into().unwrap());
        let i2 = i32::from_be_bytes(buf[5..=8].try_into().unwrap());
        if t == 'I' {
            let time = i1;
            let pennies = i2;
            prices.insert(time, pennies);
            println!("inserting at {} value {}", time, pennies);
        } else if t == 'Q' {
            let mintime = i1;
            let maxtime = i2;

            let mut sum:i64 = 0;
            let mut count:i64 = 0;
            if mintime > maxtime {
            } else {
                for (_time, price) in prices.range(mintime..=maxtime) {
                    sum += *price as i64;
                    count += 1;
                }
            }
            let mean: i64 = if count != 0 { sum / count } else { 0 };
            let mean:i32 = mean as i32;
            let response = mean.to_be_bytes();
            println!("response {}", mean);
            stream.write_all(&response)?;
        } else {
            panic!("unknown op");
        }
    }
}

fn main() -> std::io::Result<()> {
    println!("Hello, world!");

    let listener = std::net::TcpListener::bind("0.0.0.0:20345")?;

    for stream in listener.incoming() {
        println!("got connection");
        if let Ok(stream) = stream {
            std::thread::spawn(move || {
                let _ = handle_thread(stream);
                println!("connection ded");
            });
        }
    }

    Ok(())
}
