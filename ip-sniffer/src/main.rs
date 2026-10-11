// Import libraries
use std::env;
use std::io::{self, Write};
use std::net::{IpAddr, TcpStream};
use std::process;
use std::str::FromStr;
use std::sync::mpsc::{channel, Sender};
use std::thread;

// Max represents the highest possible TCP Port number.
const MAX: u16 = 65535;

// Define our instance variables for the Arguments object.
struct Arguments {
    flag: String,
    ipaddr: IpAddr,
    threads: u16,
}

// Implements a new function for arguments that returns either just the IP Address, or checks the flags passed as arguments, and then returns the IP Address plus the threads inputted.
impl Arguments {
    fn new(args: &[String]) -> Result<Arguments, &'static str> {
        // agrs[0] is the program name: ip-sniffer, and anything after are the flags, e.g args[1]. This if statement just makes sure something is passed in whether that be an IP Address or flags.
        if args.len() < 2 {
            return Err("Not enough arguments");
        } else if args.len() > 4 {
            return Err("too many arguments");
        }

        // Assign the first real argument to f.
        let f = args[1].clone();
        // We borrow f and attempt to parse it to an IP Address from a string, and if it converts to an IP Address without error, assign it to ipaddr.
        if let Ok(ipaddr) = IpAddr::from_str(&f) {
            // Return arguments with no flag, the IP Address, and the default threads which is 4.
            return Ok(Arguments {
                flag: String::from(""),
                ipaddr,
                threads: 4,
            });
            // else, we check the flags passed in to arguments.
        } else {
            // Same idea as above, take the first real argument, assign it to flag.
            let flag = args[1].clone();
            // Check if the user typed '-h' or '-help AND if it was valid, meaning we have to check if the program name was passed, and the flag, if yes, print the usage menu.
            if (flag.contains("-h") || flag.contains("-help")) && args.len() == 2 {
                println!(
                    "Usage: -j to select how many threads you want
                \r\n             -h or -help to show this help message"
                );
                // This just returns a code so that we can properly exit the program later.
                return Err("help");
                // This is if the user types say './ip-sniffer -h -j'. That obviously wouldn'ty work, so we just error out with a reason.
            } else if flag.contains("-h") || flag.contains("-help") {
                return Err("too many arguments");
                // Now we check what to do if the user types '-j'.
            } else if flag.contains("-j") {
                // The match keyword in rust, is basically just a fancy switch case with more capabilities. Here we are saying if the reference from args[3] converted from a string
                // to an IP Address returns as a success, then assign ipaddr that IP Address. If it returns as an error, error out with a reason.
                let ipaddr = match IpAddr::from_str(&args[3]) {
                    Ok(s) => s,
                    Err(_) => return Err("not a valid IPADDR; must be IPv4 or IPv6"),
                };
                // Same thing here, instead, we check if args[2] parses correctly as an u16. If it returns as a success, assign that u16 value to threads. If not error out with a reason.
                let threads = match args[2].parse::<u16>() {
                    Ok(s) => s,
                    Err(_) => return Err("failed to parse thread number"),
                };
                // If both of those matches return as a success, return Arguments with the assigned threads, the flag, and the assigned ipaddr.
                return Ok(Arguments {
                    threads,
                    flag,
                    ipaddr,
                });
                // Else, if they return as an error, error out with a reason.
            } else {
                return Err("invalid syntax");
            }
        }
    }
}

// Here we are creating a function to scan the ports on the given IP Address, this takes a thread as input, a starting port, an IP Address, and how many threads to put to work.
fn scan(tx: Sender<u16>, start_port: u16, addr: IpAddr, num_threads: u16) {
    // We have to add 1 to the starting port since the for loop that calls this function starts at 0, and there is no port 0.
    let mut port: u16 = start_port + 1; // first iteration example: port = 1

    // Here we loop until we approach the maxinmum port number.
    loop {
        // We use match again to connect to an IP Address at a certain port, if it returns as a success, we print a dot, to indicate to the user that a port was scanned and is open.
        match TcpStream::connect((addr, port)) {
            // first iteration example: addr = 192.168.1.1, port = 1
            Ok(_) => {
                print!(".");
                // This just prints it to the console.
                io::stdout().flush().unwrap();
                // We send the port through the channel we setup in main later, where rx the recieves the port value.
                tx.send(port).unwrap(); // first iteration example: port = 1
            }
            // If the port is closed, we acknowledge the connection error, ignore it, and keep looping.
            Err(_) => {}
        }

        // Here we check if we are going to hit the max number of threads, and if we do, we break out and stop.
        if (MAX - port) <= num_threads {
            // first iteration example: 65535 - 1 = 65534 <= 1000 = false.
            break;
        }

        // Here we jump to the next port, so that the next loop can check if it is open or closed.
        port += num_threads; // first iteration example: port = 1 + 1000, port = 1001
    }
}

// This is the main entry point for our program.
fn main() {
    // Assign argrs to a vector of strings, collected by env library.
    let args: Vec<String> = env::args().collect();

    // First argument is always the program name.
    let program = args[0].clone();

    // Create a new arguments object using the new function we created above. unwrap_or_else is a fancy verison of the match function we used earlier, essentially it just assigns
    // arguments with the flag, threads, and ipaddr if it succeeds, but if it returns err, it runs this bit of code instead.
    let arguments = Arguments::new(&args).unwrap_or_else(|err| {
        // If the error code was help, the program didn't fail, it just printed the usage menu, and exited cleanly.
        if err.contains("help") {
            process::exit(0);
            // If the error code was different, exit with an error, and print the error code.
        } else {
            eprintln!("{} problem parsing arguments: {}", program, err);
            process::exit(1);
        }
    });

    // Take the threads returned by arguments and assign it to num_threads.
    let num_threads = arguments.threads;

    // Same thing with IP Address.
    let addr = arguments.ipaddr;

    // Here we open up a channel for threads to work. tx is for sending, and rx is for recieving.
    let (tx, rx) = channel();

    // Loop through all the threads returned by arguments.
    for i in 0..num_threads {
        // Copy tx into another variable called tx.
        let tx = tx.clone();

        // Spawn a thread for each iteration, and call the scan function we made earlier.
        // The move keyword allows the thread to take ownership of the variables, that way it can be in scope.
        thread::spawn(move || {
            scan(tx, i, addr, num_threads);
        });
    }

    // Assign an empty vector to out.
    let mut out = vec![];

    // This is just us, allowing the channel to close.
    drop(tx);

    // Loop through all the recieved ports and add them to out.
    for p in rx {
        out.push(p);
    }

    println!("");
    // Sort the ports so they appear in order.
    out.sort();

    // For each port, print it to the console.
    for v in out {
        println!("{} is open", v);
    }
}
