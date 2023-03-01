use criterion::{black_box, criterion_group, criterion_main, Criterion, async_executor};

use tiny_zipper::zipper::file_buf_reader::*;
use tokio::io::AsyncWriteExt;
use tokio::sync::watch;

async fn test_func(){
            // generate a 1GB file for test
        // read it with 1KB buffer 
        // calculate the time cost and throughput

        println!("start to generate the test file");
        const TEST_FILE_SIZE : usize = 100 * 1024 * 1024;

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
        let mut file_reader = AsyncFileReader::new("test_file",1*1024*1024,tx).await.unwrap();

        let read_num = 4;

        // start the timer
        let start = std::time::Instant::now();

        // spwan the async read task
        let sender_handle = tokio::spawn(async move {
            file_reader.read().await;
            for _ in 0..read_num {
                file_reader.send_eof();
            }
            assert_eq!(file_reader.get_read_size(),TEST_FILE_SIZE);
        });

        // create read_num consumers to raed the data from the queue
        let mut reciver_handles = Vec::new();
        for _ in 0..read_num {
            let mut curr_rx = rx.clone();
            let reciver_handle =    tokio::spawn(async move{
                // let mut read_size = 0;
                // let mut order = 0 ;
                while curr_rx.changed().await.is_ok() {
                    //assert_eq!(order,curr_rx.borrow().order);
                    let read_buf = curr_rx.borrow();
                    if read_buf.read_size == 0 {
                        break;
                    }
                    // check the data
                    for i in 0..read_buf.read_size {
                        assert_eq!(read_buf.buf[i],19);
                    }
                }
                //println!("read_size: {},expected_file_size: {} ",read_size,TEST_FILE_SIZE);
                //assert_eq!(read_size,TEST_FILE_SIZE);
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

fn un_async_func()
{
    //println!("????");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(test_func());
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fib 20", |b| b.iter(|| un_async_func()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);