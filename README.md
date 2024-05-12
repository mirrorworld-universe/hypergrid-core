<p align="center">
  <a href="https://worldstore.mirrorworld.fun/" target="_blank"><img alt="Sonic - The First SVM Gaming Chain powered by HyperGrid" title="Sonic - The First SVM Gaming Chain powered by HyperGrid" src="https://pbs.twimg.com/media/GMZ287_XgAARYEU?format=jpg&name=medium" width="100%">
  </a>
</p>

<font face='微软雅黑' color=#ff0000 size=14>我是正文</font>

[![Live on Mainnet](https://img.shields.io/badge/Sonic%20Explorer-Live%20on%20HyperGrid-blue)](https://explorer.sonic.game)
[![Live on Mainnet](https://img.shields.io/badge/Sonic%20Faucet-Live%20on%20HyperGrid-blue)](https://faucet.sonic.game)
[![Live on Mainnet](https://img.shields.io/badge/Sonic%20Mint-Live%20on%20HyperGrid-blue)](https://mint.sonic.game)
[![Follow Us!](https://img.shields.io/twitter/follow/zksync?color=%238C8DFC&label=Follow%20%40SonicSVM&logo=data%3Aimage%2Fsvg%2Bxml%3Bbase64%2CPHN2ZyB3aWR0aD0iNDMiIGhlaWdodD0iMjUiIHZpZXdCb3g9IjAgMCA0MyAyNSIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4KPHBhdGggZmlsbC1ydWxlPSJldmVub2RkIiBjbGlwLXJ1bGU9ImV2ZW5vZGQiIGQ9Ik00Mi42NTM5IDEyLjQ5MTVMMzAuODM3OCAwLjcxNjc0M1Y5LjM0TDE5LjEwNTUgMTcuOTczOUwzMC44Mzc4IDE3Ljk4MlYyNC4yNjYyTDQyLjY1MzkgMTIuNDkxNVoiIGZpbGw9IiM0RTUyOUEiLz4KPHBhdGggZmlsbC1ydWxlPSJldmVub2RkIiBjbGlwLXJ1bGU9ImV2ZW5vZGQiIGQ9Ik0wLjk5ODA0NyAxMi40ODcyTDEyLjgxNDEgMjQuMjYxOVYxNS43MDhMMjQuNTQ2NSA3LjAwNDdMMTIuODE0MSA2Ljk5NjY0VjAuNzEyNDYzTDAuOTk4MDQ3IDEyLjQ4NzJaIiBmaWxsPSIjOEM4REZDIi8%2BCjwvc3ZnPgo%3D&style=flat)](https://twitter.com/SonicSVM)  




## Introduction

A comprehensive demonstration that showcases the ability to read and write data accounts from the Solana base layer, with interactive modifications involving data on the Grids. The writable data will be generated by our program deployed on the base layer. This demo will integrate the functionalities of reading program data, reading data accounts, and read/write operations into a single, cohesive presentation.We will implement this demo in a GitHub repository with three layers:

* Base: Code for the program/smart contract deployed on the Solana devnet.

* App: Code that interfaces with Sonic through HyperGrid, written in Rust or TypeScript.

* Docs: A README.md file that describes the nature of the demo. This file should also contain the results of stress testing.  


## Demo Video

We have produced two videos to demonstrate the features of our product:

See [Read-Demo](https://github.com/mirrorworld-universe/hypergrid-integrated-demo/blob/main/Docs/read-demo.mp4)  

See [Write-Demo](https://github.com/mirrorworld-universe/hypergrid-integrated-demo/blob/main/Docs/write-demo.mp4)
    
[![Alt text](https://github.com/mirrorworld-universe/hypergrid-integrated-demo/blob/main/Docs/read-demo.mp4.jpg)](https://github.com/mirrorworld-universe/hypergrid-integrated-demo/tree/main/Docs/)

  
## Quickstart

See [https://internaldocs.sonic.game/](https://internaldocs.sonic.game/)    


## Testing Report

1. Report Overview
    * Project Name: Sonic HyperGrid Integrated Demo
    * Testing Period: May 6, 2024, to May 9, 2024
    * Purpose of Testing: 
      Perform thorough testing to validate the functionality, security, and performance of the read-write synchronization mechanism, including unit testing, integration testing, and real-world simulation testing.
      The test will be focusing on two parts:  
      (1) The correctness of the state syncing, for instance the non-existence state sync, the out-date-state syncing, the existing data override, etc.  
      (2) The performance of the state syncing, using timestamp or similar metrics to measure the normal transaction without accessing the base layer data and transaction accessing the base layer data.  

2. Test Environment
    * Hardware Environment: 32-core CPU, 64GB RAM, 1T SSD
    * Software Environment: Ubuntu 22.04 LTS OS

3. Testing Strategy
    * Unit Testing: Focus on individual components or modules.
    * Integration Testing: Focus on combined parts of the application to determine if they function together correctly.
    * Real-World Simulation Testing: Testing the application in a simulated real-world environment.

4. Testing Results:
   * [[ Testing Results ]](https://docs.google.com/spreadsheets/d/1PPERkiRJ1vpOxsKrMeHiwATo8Bc77SPYZbyhluacBPU/)  
  
## Disclaimer

This project is a Layer 2 rollup solution for the Solana devnet, and it adopts the core algorithms and operation mechanisms of the Solana devnet. Therefore, all claims, content, designs, algorithms, estimates, roadmaps, specifications, and performance measurements described herein not only adhere to and inherit the statements made by Solana Labs, Inc. regarding the Solana project, but also reflect the good faith efforts of MirrorWorld ("MW"). Readers are responsible for verifying their accuracy and truthfulness. Additionally, nothing in this project constitutes an invitation for investment.

Any content or developer resources provided by MW are for educational and inspirational purposes only. MW does not encourage, induce, or endorse the deployment, integration, or use of any such applications (including the code comprising the Solana blockchain protocol) in violation of applicable laws or regulations and hereby prohibits any such deployment, integration, or use. This includes the violation of export control or sanctions laws of the United States or any other applicable jurisdiction by the reader (a), residing in or being a resident of a country or territory subject to comprehensive sanctions administered by the U.S. Office of Foreign Assets Control (OFAC) (b), or being or working on behalf of a Specially Designated National (SDN) or a person subject to similar blocking or denied party prohibitions (c).

Readers should be aware that U.S. export control and sanctions laws prohibit U.S. persons (and other persons subject to such laws) from transacting with individuals in certain countries and territories or who are on the SDN list. Therefore, individuals risk violating U.S. export controls and sanctions laws by transacting with others using any of the code contained in this repository or its derivatives, who may be sanctioned individuals.
