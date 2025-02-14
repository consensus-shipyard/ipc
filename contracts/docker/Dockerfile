# Use an official Node image (using Node 22 here, but consider using an LTS version if preferred)
FROM node:22

# Install necessary packages.
RUN apt-get update && apt-get install -y curl

# Install PNPM globally.
RUN npm install -g pnpm

# Install Foundry (for Anvil). Specifying a version if needed.
RUN curl -L https://foundry.paradigm.xyz | bash && \
    /root/.foundry/bin/foundryup --install 0.3.0

# Add Foundry binaries to PATH.
ENV PATH="/root/.foundry/bin:$PATH"

# Set working directory.
WORKDIR /app

# Copy the entire repository.
COPY . .

# Install Node dependencies (skip lifecycle scripts if desired).
RUN npm install --ignore-scripts

# (Optional) Pre-compile contracts.
RUN npx hardhat compile

# Set the commit SHA from a build argument.
ARG COMMIT_SHA
ENV COMMIT_SHA=${COMMIT_SHA}

# Copy the entrypoint script and make it executable.
COPY docker/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

# Expose the port for Anvil.
EXPOSE 8545

# Run the container with the entrypoint.
CMD ["/entrypoint.sh"]
