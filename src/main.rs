use proyecto1::mypthreads::*;

fn main() {
    println!("probando mi biblioteca...");

    let _t1 = my_thread_create(dummy, SchedulerType::RoundRobin)
        .expect("no se pudo crear hilo");

    // ... luego podrías llamar my_thread_yield_();
}

fn dummy() {
    println!("soy un hilo!");
}
