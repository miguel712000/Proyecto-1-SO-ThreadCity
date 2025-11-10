use proyecto1::mypthreads::{my_thread_create, my_thread_run_once, SchedulerType};

fn hilo_a() {
    println!("Hola, soy el hilo A");
}

fn hilo_b() {
    println!("Hola, soy el hilo B");
}

fn main() {
    let t1 = my_thread_create(hilo_a, SchedulerType::RoundRobin).unwrap();
    let t2 = my_thread_create(hilo_b, SchedulerType::RoundRobin).unwrap();

    my_thread_run_once(t1);
    my_thread_run_once(t2);
}
