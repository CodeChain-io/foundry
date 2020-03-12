IBC relayer and scenario
=========================

This directory contains IBC relayer implementation and IBC demo scenario script.

## Before start

1. Please run `yarn install`. It will install dependencies.
2. Please run `cp .env.default .env`

## How to run chains

Run `yarn run runChains`

## How to run a scenario

Run `yarn run scenario`. It will create light clients, a connection, and a channel.
Finally, it will send a packet.

## Print debug log

Please use `DEBUG` environment variable.
If you want to print all debug log,
please use "\*" for the `DEBUG` environment variable, like: `DEBUG="*"`

For example you can run like this: `DEBUG="*" yarn run runChains`