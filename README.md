# gym-backend

This is the backend for the **Gym Occupancy** Project.

Employing Asynchronous Rust, this application does 3 things:

1. Web Scraper
2. Custom Firebase API
3. Prediction using the Weighted KNN Regressor Algorithm
4. Sleeper - Async Sleep until required adhering to the gym schedule.

All of these processes (Including the Sleeping between intervals) happen concurrently such that there is minimal discrepancy between data collection, processing, generation and uploading. 

The code is robust such that failures from the website or network do not entirely stop the process from being conducted. If any errors occur during runtime, it will append the log message to a file (<Week \Start>.data) and continue. 

## Web Scraper

The Web Scraping is mainly conducted with the extensive use of RegEx. 

Not only do we scrape the Occupancy %, but we also scrape the Schedule of the Gym Opening Times which is all stored within their respective structs and is later sent to the Firebase Real-Time Database via our Firebase API.

## Firebase API

The custom Firebase API communicates with the Firebase REST API and automatically authenticates itself with the serviceAccount.json key using Json Web Tokens. It is written in a manner which allows for it to communicate with other Firebase databases as well. 

## Weighted KNN Regressor

A modified KNN Classifier algorithm to be able to perform a regression. 

Weights are added to give priority to newer and closer data points. This makes the algorithm more reactive to seasonal changes - requiring maybe a week to be relevant.

## Sleeper

Async Sleeps for a fixed amount of time adhering to any errors and the gym opening hours. 
