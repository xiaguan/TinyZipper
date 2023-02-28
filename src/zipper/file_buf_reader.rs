use std::sync::Arc;
use tokio::{sync::watch, io::AsyncReadExt};

#[derive(Debug)]
pub struct ReadBuf{
    pub order : usize,
    pub buf : Vec<u8>,
    pub read_size : usize,
}
// use tokio lib to async read the fixed size of data from the file
// then put the data into a single producer multiple consumer queue
pub struct AsyncFileReader{
    file : tokio::fs::File,
    buf_size : usize,
    queue : watch::Sender<ReadBuf>,
    read_size : usize,
    read_order : usize,
}

impl AsyncFileReader {
    pub async fn new(file : &str,buf_size : usize,queue : watch::Sender<ReadBuf>) -> Option<AsyncFileReader> {
        let file = tokio::fs::File::open(file).await;
        match file {
            Ok(file) => {
                return Some(AsyncFileReader{file,buf_size,queue,read_size:0,read_order:0});
            },
            Err(e) => {
                println!("Error: {}",e);
                return None;
            }
        }
    }

    pub async fn read(&mut self) {
        let mut buf = vec![0u8;self.buf_size];
        let mut read_size = 0;
        loop {
            match self.file.read(&mut buf).await {
                Ok(size) => {
                    if size == 0 {
                        break;
                    }
                    read_size += size;
                    let read_buf = ReadBuf{order:self.read_order,buf:buf.clone(),read_size:size};
                    self.queue.send(read_buf).unwrap();
                    self.read_order += 1;
                },
                Err(e) => {
                    println!("Error: {}",e);
                    break;
                }
            }
        }
        self.read_size = read_size;
    }

    // let the consumer know the file is read completely
    pub fn send_eof(&mut self) {
        let read_buf = ReadBuf{order:self.read_order,buf:vec![],read_size:0};
        if self.queue.send(read_buf).is_err() {
            println!("send_eof: failed to send EOF");
        }
    }

}

// add test for the async file reader
#[cfg(test)]
mod tests{
    use std::cell::RefCell;

    use crate::zipper::file_buf_reader::*;
    use tokio::{sync::watch, io::AsyncWriteExt};

    #[tokio::test]
    async fn read_test_file_and_check(){

        let expected_file = String::from("Hello World!Some thing just like this !");
        let expected_file_size = expected_file.len();

        // use std lib to write the test file,and flush the data to the disk
        let mut file = tokio::fs::File::create("test_file").await.unwrap();
        file.write_buf(&mut expected_file.as_bytes()).await.unwrap();
        file.flush().await.unwrap();


        let (tx,mut rx) = watch::channel(ReadBuf{order:0,buf:vec![],read_size:0});
        let mut file_reader = AsyncFileReader::new("test_file",2,tx).await.unwrap();

        // spwan the async read task
        let sender_handle = tokio::spawn(async move {
            file_reader.read().await;
        });

        // create a consumer to raed the data from the queue
        let reciver_handle =    tokio::spawn(async move{
            let mut read_size = 0;
            let mut read_str = String::new();
            let mut order = 0 ;
            while rx.changed().await.is_ok() {
                assert_eq!(order,rx.borrow().order);
                let read_buf = rx.borrow();
                if read_buf.read_size == 0 {
                    break;
                }
                read_size += read_buf.read_size;
                order += 1;
                // push read_buf.read_size length of data into the read_str
                read_str.push_str(std::str::from_utf8(&read_buf.buf[0..read_buf.read_size]).unwrap());
            }
            println!("read_size: {},expected_file_size: {} ",read_size,expected_file_size);
            assert_eq!(read_size,expected_file_size);
            println!("read_str: {},expected_file: {}",read_str,expected_file);
            assert_eq!(read_str,expected_file);
        });

        // wait for the async read task to complete
        sender_handle.await.unwrap();
        // wait for the consumer to complete
        reciver_handle.await.unwrap();
        
    }

    #[tokio::test]
    async fn bench_test(){
        // generate a 1GB file for test
        // read it with 1KB buffer 
        // calculate the time cost and throughput


        const TEST_FILE_SIZE : usize = 1024 * 1024 * 1024;

        let std_start = std::time::Instant::now();
        let mut file = tokio::fs::File::create("test_file").await.unwrap();
        let buf = vec![19u8;1024];
        let mut write_size = 0;
        while write_size < TEST_FILE_SIZE {
            file.write_buf(&mut buf.as_slice()).await.unwrap();
            write_size += buf.len();
        }
        file.flush().await.unwrap();
        let std_end = std::time::Instant::now();
        println!("std write file cost: {:?}",std_end.duration_since(std_start));

        let (tx,rx) = watch::channel(ReadBuf{order:0,buf:vec![],read_size:0});
        let mut file_reader = AsyncFileReader::new("test_file",1024,tx).await.unwrap();

        let read_num = 4;

        // start the timer
        let start = std::time::Instant::now();

        // spwan the async read task
        let sender_handle = tokio::spawn(async move {
            file_reader.read().await;
            for _ in 0..read_num {
                file_reader.send_eof();
            }
        });

        // create read_num consumers to raed the data from the queue
        let mut reciver_handles = Vec::new();
        for _ in 0..read_num {
            let mut curr_rx = rx.clone();
            let reciver_handle =    tokio::spawn(async move{
                let mut read_size = 0;
                let mut order = 0 ;
                while curr_rx.changed().await.is_ok() {
                    assert_eq!(order,curr_rx.borrow().order);
                    let read_buf = curr_rx.borrow();
                    if read_buf.read_size == 0 {
                        break;
                    }
                    read_size += read_buf.read_size;
                    order += 1;
                }
                //println!("read_size: {},expected_file_size: {} ",read_size,TEST_FILE_SIZE);
                assert_eq!(read_size,TEST_FILE_SIZE);
            });
            reciver_handles.push(reciver_handle);
        }
        sender_handle.await.unwrap();
        for reciver_handle in reciver_handles {
            reciver_handle.await.unwrap();
        }

        // stop the timer
        let end = std::time::Instant::now();
        let time_cost = end.duration_since(start);
        let throughput = TEST_FILE_SIZE as f64 / time_cost.as_secs_f64();
        println!("time_cost: {:?},throughput: {} MB/s",time_cost,throughput / 1024.0 / 1024.0);

    }
}
