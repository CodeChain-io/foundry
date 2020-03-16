IBC relayer and scenario
=========================

This directory contains IBC relayer implementation and IBC demo scenario script.

## Before start

1. Prepare the node version 12, yarn and rust development environment.
1. Please run `yarn install`. It will install dependencies.
1. Please run `cp .env.default .env`

## Running an IBC scenario

1. Run two foundry chains using `yarn run runChains`. Since they will use 13485, 13486, 18080 and 18081 port, please make sure the ports are available.
1. When a "Chains are running!" is printed, run relayer and scenario script.
   1. Run a relayer using `yarn run relayer`.
   1. Run a scenario script using `yarn run scenario`.
1. Create light clients connection, and channel in the scenario script. The scenario script has an interactive user interface for you.
1. Send a packet using the scenario script.

## Print debug log

Please use `DEBUG` environment variable.
If you want to print all debug log,
please use "\*" for the `DEBUG` environment variable, like: `DEBUG="*"`

For example you can run like this: `DEBUG="*" yarn run runChains`
