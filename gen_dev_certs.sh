#!/bin/bash

CERT_DIR="./cfg/cert"

mkdir -p "${CERT_DIR}" || exit 2
pushd "bin/mw_certgen/"

# ROOT CA
cargo r -- -d "../../${CERT_DIR}" gen-root-ca \
    root.ca.cert.der root.ca.key.der &&

# Sub-CAs
cargo r -- -d "../../${CERT_DIR}" gen-sub-ca \
    --ca root.ca.cert.der --ca-key root.ca.key.der \
    hosts.ca.cert.der hosts.ca.key.der &&
cargo r -- -d "../../${CERT_DIR}" gen-sub-ca \
    --ca root.ca.cert.der --ca-key root.ca.key.der \
    auths.ca.cert.der auths.ca.key.der &&
cargo r -- -d "../../${CERT_DIR}" gen-sub-ca \
    --ca root.ca.cert.der --ca-key root.ca.key.der \
    apps.ca.cert.der apps.ca.key.der &&

# Auth Server
cargo r -- -d "../../${CERT_DIR}" gen-auth-server-cert \
    --ca auths.ca.cert.der --ca-key auths.ca.key.der \
    -n "auth00.localhost" \
    auth00.cert.der auth00.key.der &&
cargo r -- -d "../../${CERT_DIR}" gen-host-auth-server-cert \
    --ca auths.ca.cert.der --ca-key auths.ca.key.der \
    -n "auth00.localhost" \
    auth00hostauth.cert.der auth00hostauth.key.der &&
cargo r -- -d "../../${CERT_DIR}" gen-session-ca \
    --ca auth00.cert.der --ca-key auth00.key.der \
    auth00session.ca.cert.der auth00session.ca.key.der &&

# Host Server
cargo r -- -d "../../${CERT_DIR}" gen-host-server-cert \
    --ca hosts.ca.cert.der --ca-key hosts.ca.key.der \
    -n "host00.localhost" \
    host00.cert.der host00.key.der &&
cargo r -- -d "../../${CERT_DIR}" gen-host-rpc-server-cert \
    --ca hosts.ca.cert.der --ca-key hosts.ca.key.der \
    -n "host00.localhost" \
    host00rpc.cert.der host00rpc.key.der &&
cargo r -- -d "../../${CERT_DIR}" gen-host-auth-client-cert \
    --ca hosts.ca.cert.der --ca-key hosts.ca.key.der \
    host00hostauth.cert.der host00hostauth.key.der &&

# HostRPC

cargo r -- -d "../../${CERT_DIR}" gen-host-rpc-client-cert \
    --ca root.ca.cert.der --ca-key root.ca.key.der \
    hostrpc.cert.der hostrpc.key.der &&

# App

cargo r -- -d "../../${CERT_DIR}" gen-auth-client-cert \
    --ca apps.ca.cert.der --ca-key apps.ca.key.der \
    authclient.cert.der authclient.key.der &&
cargo r -- -d "../../${CERT_DIR}" gen-host-client-cert \
    --ca apps.ca.cert.der --ca-key apps.ca.key.der \
    hostclient.cert.der hostclient.key.der &&

exit 0
