# Task Scheduler

## How it works

This task scheduler runs in `master` or `slave` mode to scale horizontally. Both master and slave do this things:

- API Interface
- Act as Worker to process tasks

While only master accomplish this other things:

- Store tasks and workers in SQLite DB (it's instantiated in /tmp)
- Run the scheduler in order to execute tasks in the right time
- Distribute jobs across the workers network, depending on availability
- Collect tasks that are coming from the slaves API interface

## Quick Start

Two args are expected: `{network_address}` and `{master|slave}`.

```
./target/release/fizzbuzz-task-scheduler {network_address} {master|slave}
```

Example: `cargo build --release && ./target/release/fizzbuzz-task-scheduler 127.0.0.1:8080 master`

## API Interface

### Set Task

```bash
curl -X "POST" "http://127.0.0.1:8081/task/set" \
     -H 'Content-Type: application/json; charset=utf-8' \
     -d $'{
  "kind": "FizzBuzz",
  "when": 1669369358
}'
```

### List Tasks

```bash
curl "http://127.0.0.1:8080/task?status=Completed" \
     -H 'Content-Type: application/json; charset=utf-8' \
     -d $'{
  "kind": "FizzBuzz",
  "when": 1003
}'
```

### Get Single task

```bash
curl "http://127.0.0.1:8080/task/T1QfaGU8DBQCfEYR?status=Pending" \
     -H 'Content-Type: application/json; charset=utf-8' \
     -d $'{
  "kind": "FizzBuzz",
  "when": 1003
}'
```

### Delete single task

```bash
curl -X "POST" "http://127.0.0.1:8080/task/delete/jZSmtAe81ng40nZG?status=Pending" \
     -H 'Content-Type: application/json; charset=utf-8' \
     -d $'{
  "kind": "FizzBuzz",
  "when": 1003
}'
```

## Assignment

We’re going to build a small task scheduling service. The service consists of an API listener, which accepts HTTP API calls, and a worker which executes tasks of different types. There are 3 task types: "Fizz", "Buzz", and "FizzBuz".

For "Fizz" tasks, the worker should pause the task for 3 seconds, and then print "Fizz {task_id}".
For "Buzz" tasks, the worker should pause the task for 5 seconds, and then print "Buzz {task_id}".
For "FizzBuzz" tasks, the worker should pause the task for 15 seconds, and then print "Fizz Buzz {task_id}".
Requirements

Expose an API that can:
Create a task of a specific type and execution time, returning the task's ID
Show a list of tasks, filterable by their state (whatever states you define) and/or their task type
Show a task based on its ID
Delete a task based on its ID
The tasks must be persisted into some external data store (your choice).
Process each task only once and only at/after their specified execution time.
Support running multiple instances of your code in parallel, such that a cloud deployment could be horizontally scaled.
Open a PR against an empty repository that you create.
General guidelines

Please don't spend more than 2-3 hours on this.
Feel free to use common libraries, tools, and open source software.
It's OK to skip things if you run out of time, or if you think something is not essential for v1. Just make sure to document these decisions in the pull request.
Don't try to be fancy, just write code you would normally write when working on a similar task.
If you don't know or unsure about something, it's fine to say so.
