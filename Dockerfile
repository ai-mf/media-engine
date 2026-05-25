FROM ubuntu:22.04

# Install dependencies
RUN apt-get update && apt-get install -y \
    ffmpeg \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy AIMF binary
COPY target/release/aimf /usr/local/bin/aimf
RUN chmod +x /usr/local/bin/aimf

# Default command
ENTRYPOINT ["aimf"]
CMD ["--help"]
