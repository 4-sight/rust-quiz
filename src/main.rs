// File system module.
use std::fs;
// `self` allows us to reference the `io` module later.
// Read and Write trait let us read the quiz file and write to `stdout`.
use std::io::{self, Read, Write};
// `mpsc` stands for multiple producer and single consumer.
// This is our channel between threads.
use std::sync::mpsc;
// import the thread module.
use std::thread;
// Duration is how we time the user.
use rand::prelude::*;
use std::time::Duration;

fn main() {
    let time_limit = 4;
    // Random number generator used to shuffle questions
    let mut rng = rand::thread_rng();

    let quiz_tsv_filename = "quiz.tsv";
    let mut quiz_file = fs::File::open(quiz_tsv_filename).unwrap();
    let mut buf = String::new();
    quiz_file.read_to_string(&mut buf).unwrap();

    let number_of_questions = buf.lines().count();

    let mut questions = buf
        // .lines() iterates over the buffer lines
        .lines()
        // Break each line into q & a tuples
        .map(|line| {
            let mut q_a = line.split(',').map(|s| s.to_string());
            let question = q_a.next().expect("No question found.");
            let answer = q_a.next().expect("No answer found.");
            (question, answer)
        })
        // transform iteration into vec
        .collect::<Vec<(String, String)>>();

    // Shuffle to randomize question order
    questions.shuffle(&mut rng);
    let shuffled_questions = questions;

    // Iterate over questions and pass each to test_question
    let score = shuffled_questions
        .iter()
        .map(|(question, answer)| test_question(&question, &answer, time_limit))
        // Now we want to count the correct answers by filtering out the false bool
        .take_while(|o| o.is_some())
        // We have an invariant that any value below the `take_while` must be Some,
        // so we can unwrap.
        // (You should still assume coder error could happen so `unwrap` is probably a bad idea)
        .map(|o| o.unwrap())
        // Now we want to count the correct answers by filtering out the false bool.
        .filter(|p| *p)
        // `count` will count the number of elements that reach it.
        // In this case we are counting the number of true values or correct answers to the questions.
        .count();

    println!("Score: {} / {}", score, number_of_questions);
}

fn test_question(question: &str, answer: &str, time_limit: u32) -> Option<bool> {
    println!("{}", question);
    // We have to flush the question to the display, otherwise
    // it may not appear because Rust is trying to optimize display calls.
    io::stdout().flush().expect("Failed to flush the buffer");

    // Set up our transmitter and receiver to use between threads.
    let (transmitter, receiver) = mpsc::channel();

    // Spawn a thread with the user input code
    thread::spawn(move || {
        // Read the user input into a buffer
        let mut buffer = String::new();
        io::stdin()
            .read_line(&mut buffer)
            .expect("Failed to read user input");

        let buffer = buffer.trim().to_string();
        // And send the buffer into the transmitter
        // This is how we get the buffer out of the thread
        transmitter.send(buffer).expect("Failed to send user input");
    });

    // The spawned thread doesn't block so we jump straight to this line of code
    // Here we want the receiver to wait for the given time or until the user's
    // answer is provided
    receiver
        .recv_timeout(Duration::new(time_limit as u64, 0))
        .or_else(|o| {
            println!("\n    Time's up!");
            Err(o)
        })
        .ok()
        .map(|buffer| buffer == answer)
}
