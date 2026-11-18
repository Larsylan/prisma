# Environnement Linux de l'enregistrement : `scripts/demo/record.sh` le construit et l'utilise.
FROM ubuntu:22.04
ENV DEBIAN_FRONTEND=noninteractive LANG=en_US.UTF-8
RUN apt-get update && apt-get install -y build-essential curl pkg-config libssl-dev libwebkit2gtk-4.1-dev \
    libayatana-appindicator3-dev librsvg2-dev libasound2-dev libdbus-1-dev libxdo-dev \
    xvfb xcompmgr feh xdotool ffmpeg dbus dbus-x11 fonts-roboto python3 locales \
    && locale-gen en_US.UTF-8 && rm -rf /var/lib/apt/lists/*
RUN curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
ENV PATH=/root/.cargo/bin:$PATH
