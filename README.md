## OXIS

A minimal file shredder utility written in Rust.

### Overview
OXIS is a lightweight file shredder that securely deletes files by overwriting them with random data.
This utility is designed to be simple and efficient. Its usecase is to permanently erase sensitive information.

### Features
* Multiple overwrites: effectively overwrites files multiple times before deletion to prevent recovery.


### Pros
* Minimal dependencies: only relies on the `getrandom` crate
* lightweight: (not yet) 

### State of the Project
The project is currently functional, with the core shredding functionality implemented. However, it still is under development. shredding only work one file at a time.

### Usage
To use OXIS, simply compile the Rust code and run the resulting executable. You can then specify the files you want to shred as command-line arguments.
command : $ oxis filename
