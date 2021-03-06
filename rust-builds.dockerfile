FROM rust:latest

ENV UNATTENDED=1 \
    OSX_VERSION_MIN=10.15 \
    PKG_FILE=MacOSX10.15.sdk.tar.xz \
    PKG_CONFIG_ALLOW_CROSS=1

WORKDIR /usr/src/

RUN apt-get update && \
    apt-get install -qqy --no-install-recommends \
    clang \
    cmake \
    g++ \
    gcc \
    libgmp-dev \
    libmpc-dev \
    libmpfr-dev \
    zlib1g-dev; \
    apt-get remove tmux

RUN git clone https://github.com/tpoechtrager/osxcross.git --depth 1 /osxcross/
COPY $PKG_FILE /osxcross/tarballs/

RUN cd /osxcross \
    && ./build.sh -y \
    && rm tarballs/$PKG_FILE

ENV PATH /osxcross/target/bin:$PATH

RUN apt-get remove \
    clang \
    cmake \
    g++ \
    gcc \
    libgmp-dev \
    libmpc-dev \
    libmpfr-dev \
    zlib1g-dev; \
    rm -rf /var/lib/apt/lists/*;

RUN rustup target add x86_64-apple-darwin; \
    rustup component add rustfmt; \
    cargo install clog-cli
