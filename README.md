# folder-lock-decrypt-android

GUI program to decrypt images from the Android app Folder Lock (com.newsoftwares.folderlock_v1)

## How it works

The "Encryption" process of the app reverses the first 111 bytes of each file. This code will revert these bytes back for all the files in the directory


## User Guide

1. Export the directory of encrypted images from the device
2. Run the code
3. Enter the path to the directory holding the encrypted files
4. Enter the path to the directory where you would like the decrypted files
5. (optional) Export decryption logs in either JSON or .txt format

## Build & Run

### Build

To build an executable file: 

1. Ensure that Rust is installed
2. `git clone https://github.com/c-sleuth/folder-lock-decrypt-android.git`
3. `cd folder-lock-decrypt-android`
4. `cargo build --release`

### Run 

To run the code without building:

1. Ensure that Rust is installed
2. `git clone https://github.com/c-sleuth/folder-lock-decrypt-android.git`
3. `cd folder-lock-decrypt-android`
4. `cargo run`

