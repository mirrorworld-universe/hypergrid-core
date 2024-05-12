<p align="center">
  <a href="https://worldstore.mirrorworld.fun/" target="_blank"><img alt="Sonic - The First SVM Gaming Chain powered by HyperGrid" title="Sonic - The First SVM Gaming Chain powered by HyperGrid" src="https://pbs.twimg.com/media/GMZ287_XgAARYEU?format=jpg&name=medium" width="100%">
  </a>
</p>


[![Live on Mainnet](https://img.shields.io/badge/Sonic%20Explorer-Live%20on%20HyperGrid-blue)](https://explorer.sonic.game)
[![Live on Mainnet](https://img.shields.io/badge/Sonic%20Faucet-Live%20on%20HyperGrid-blue)](https://faucet.sonic.game)
[![Live on Mainnet](https://img.shields.io/badge/Sonic%20Mint-Live%20on%20HyperGrid-blue)](https://mint.sonic.game)
[![Follow Us!](https://img.shields.io/twitter/follow/zksync?color=%238C8DFC&label=Follow%20%40SonicSVM&logo=data%3Aimage%2Fsvg%2Bxml%3Bbase64%2CPHN2ZyB3aWR0aD0iNDMiIGhlaWdodD0iMjUiIHZpZXdCb3g9IjAgMCA0MyAyNSIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4KPHBhdGggZmlsbC1ydWxlPSJldmVub2RkIiBjbGlwLXJ1bGU9ImV2ZW5vZGQiIGQ9Ik00Mi42NTM5IDEyLjQ5MTVMMzAuODM3OCAwLjcxNjc0M1Y5LjM0TDE5LjEwNTUgMTcuOTczOUwzMC44Mzc4IDE3Ljk4MlYyNC4yNjYyTDQyLjY1MzkgMTIuNDkxNVoiIGZpbGw9IiM0RTUyOUEiLz4KPHBhdGggZmlsbC1ydWxlPSJldmVub2RkIiBjbGlwLXJ1bGU9ImV2ZW5vZGQiIGQ9Ik0wLjk5ODA0NyAxMi40ODcyTDEyLjgxNDEgMjQuMjYxOVYxNS43MDhMMjQuNTQ2NSA3LjAwNDdMMTIuODE0MSA2Ljk5NjY0VjAuNzEyNDYzTDAuOTk4MDQ3IDEyLjQ4NzJaIiBmaWxsPSIjOEM4REZDIi8%2BCjwvc3ZnPgo%3D&style=flat)](https://twitter.com/SonicSVM)




## Integrated Demo

A comprehensive demonstration that showcases the ability to read and write data accounts from the Solana base layer, with interactive modifications involving data on the Grids. The writable data will be generated by our program deployed on the base layer. This demo will integrate the functionalities of reading program data, reading data accounts, and read/write operations into a single, cohesive presentation.We will implement this demo in a GitHub repository with three layers:

* Base: Code for the program/smart contract deployed on the Solana devnet.

* App: Code that interfaces with Sonic through HyperGrid, written in Rust or TypeScript.

* Docs: A README.md file that describes the nature of the demo. This file should also contain the results of stress testing.



## Integrated Demo Project Testing Report

1. Report Overview
    * Project Name: Sonic HyperGrid Integrated Demo
    * Testing Period: May 6, 2024, to May 9, 2024
    * Purpose of Testing: 
      Perform thorough testing to validate the functionality, security, and performance of the read-write synchronization mechanism, including unit testing, integration testing, and real-world simulation testing.
      The test will be focusing on two parts:
      ** (1) The correctness of the state syncing, for instance the non-existence state sync, the out-date-state syncing, the existing data override, etc.
      ** (2) The performance of the state syncing, using timestamp or similar metrics to measure the normal transaction without accessing the base layer data and transaction accessing the base layer data.

2. Test Environment
    * Hardware Environment: 32-core CPU, 64GB RAM, 1T SSD
    * Software Environment: Ubuntu 22.04 LTS OS

3. Testing Strategy
    * Unit Testing: Focus on individual components or modules.
    * Integration Testing: Focus on combined parts of the application to determine if they function together correctly.
    * Real-World Simulation Testing: Testing the application in a simulated real-world environment.

4. Testing Results:
   * [Testing Results](https://docs.google.com/spreadsheets/d/1PPERkiRJ1vpOxsKrMeHiwATo8Bc77SPYZbyhluacBPU/ "Testing Results")






