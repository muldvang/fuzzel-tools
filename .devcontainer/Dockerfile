FROM ubuntu:noble

RUN apt-get update && apt-get install -y \
   git \
   curl \
   build-essential \
   pkg-config \
   libglib2.0-dev \
   libsecret-1-dev

USER ubuntu

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/home/ubuntu/.cargo/bin:${PATH}"