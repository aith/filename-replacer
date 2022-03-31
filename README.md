## Summary
This file replaces a string occurences in filenames and file contents throughout a directory.
Uses a temporary location to safely edit files before writing.
Case-sensitive.

## Example
The following changes occurences of 'compootur' to 'computer':  
`cargo run -- -w computer compootur `  

So that a file named `computer testing.md` with the contents:  
`I love my computer :)`  
Would be changed to `compootur testing.md` with the contents:  
`I love my compootur :)`  