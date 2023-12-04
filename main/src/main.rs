use clap::Parser;
use graph::graph::Graph;
use rand::seq::SliceRandom;
use rand::Rng;
use spinners::{Spinner, Spinners};
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::process::Command;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    #[arg(short, long, default_value_t = {"full_table".to_string()})]
    type_of_input: String,
    #[arg(short, long, default_value_t = {"in.csv".to_string()})]
    input: String,
    #[arg(short, long, default_value_t = {10})]
    number_of_tests: u32,
}

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    let arguments = Arguments::parse();
    let mut graph: Graph<u32> = Graph::<u32>::read_graph_from_file_full_table(&arguments.input);
    graph.set_zero_to_max();
    let graph = &graph;
    print!("{graph}\n");

    for i in 0..arguments.number_of_tests {
        let result = measure_execution_time(&arguments.input, || {
            return annealing(
                &graph,
                graph.number_of_vertex(),
                1000.0,
                100000,
                u32::MAX,
                0.9,
            );
        });
        let min_cost = result.1 .0;
        let path_str = format!("{:?}", &result.1 .1).replace(",", " -");
        print!("Minimum cost is {min_cost} on path {path_str}\n");
        std::fs::write(
            format!("{}_{}.out.csv", arguments.input, i),
            format!("{}, {}, {}\n", result.0.as_nanos(), result.1 .0, path_str),
        )
        .expect("Expected to write output file");
    }
}

fn measure_execution_time<T>(filename: &String, function: impl Fn() -> T) -> (Duration, T) {
    let (tx, rx) = mpsc::channel();
    let filename = filename.clone();
    thread::spawn(move || {
        let mut spinner = Spinner::new(Spinners::Pipe, "Calculating best route".into());
        let time = Instant::now();
        loop {
            let mem = Command::new("ps")
                .arg("-axm")
                .arg("-o %mem,rss,comm")
                .output()
                .expect("Expected to probe memory usage");
            let mem = String::from_utf8_lossy(&mem.stdout);
            match mem.lines().find(|l| l.contains("held_karp")) {
                Some(l) => {
                    match OpenOptions::new()
                        .append(true)
                        .open(format!("{}.out.mem", filename))
                    {
                        Ok(mut f) => {
                            f.write(format!("{}\n", l).as_bytes())
                                .expect("Expected to write output file");
                            1
                        }
                        Err(_) => {
                            std::fs::write(format!("{}.out.mem", filename), format!("{}\n", l))
                                .expect("Expected to write output file");
                            1
                        }
                    };
                }
                None => {}
            }
            match rx.try_recv() {
                Ok(_) | Err(TryRecvError::Disconnected) => {
                    spinner.stop();
                    return;
                }
                Err(TryRecvError::Empty) => {}
            }
            if time.elapsed().as_secs() >= 1800 {
                spinner.stop();
                panic!("Exceeded time limit set to 1800s!");
            }
            thread::sleep(Duration::from_millis(1));
        }
    });
    let instant = Instant::now();
    let res = function();
    let elapsed = instant.elapsed();
    let nanos = elapsed.as_nanos();
    let _ = tx.send(());
    print!("\nTime of execution: {nanos} ns\n");
    return (elapsed, res);
}

fn annealing(
    graph: &&Graph<u32>,
    n: usize,
    temperature: f32,
    era_len: u32,
    eras_n: u32,
    coolant: f32,
) -> (u32, Vec<usize>) {
    let mut rng = rand::thread_rng();
    let mut prev: Vec<usize> = (1..=n).collect();
    let vec_of_vert_numbers: Vec<usize> = (0..n).collect();
    let mut temperature = temperature;
    let mut prev_distance = graph.distance_vec(&prev);

    let time = Instant::now();
    let mut plot = Vec::<(u128, u32)>::with_capacity(era_len as usize);
    loop {
        for _ in 0..era_len {
            let mut curr = prev.clone();
            let swap_ids: Vec<usize> = vec_of_vert_numbers
                .choose_multiple(&mut rng, 2)
                .cloned()
                .collect();
            curr.swap(swap_ids[0], swap_ids[1]);
            let curr_distance = graph.distance_cycle(&curr);
            if prev_distance < curr_distance {
                let random = rand::thread_rng().gen_range(0.0..=1.0);
                let chance = std::f32::consts::E
                    .powf(-((curr_distance - prev_distance) as f32 / temperature));
                if random > chance {
                    continue;
                }
            }
            prev = curr;
            prev_distance = curr_distance;
            plot.push((time.elapsed().as_nanos(), prev_distance));
        }
        temperature = temperature * coolant;
        if temperature < 0.1 {
            break;
        }
    }
    let plot = plot
        .iter()
        .map(|x| format!("{}, {}\n", x.0, x.1))
        .reduce(|cur: String, nxt: String| cur + &nxt)
        .unwrap();
    _ = std::fs::write("plot.csv", plot);
    return (prev_distance, prev);
}
