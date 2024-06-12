# Before deploying:

- Copy `.env.template` to `.env`.
- In `.env`, fill in your own values for private key and RPC URL for the target network (e.g. for Calibrationnet).
- Install `pnpm`: `npm install -g pnpm`.

# To deploy everything run:

```bash
pnpm exec hardhat deploy
```

## To deploy only the libraries:

```bash
pnpm exec hardhat deploy-libraries
```

## To deploy only the Gateway:

```bash
pnpm exec hardhat deploy-gateway
```

## To deploy only the Gateway Actor:

```bash
pnpm exec hardhat deploy-gateway
```

## To deploy only the Registry:

```bash
pnpm exec hardhat run scripts/deploy-registry.ts
```
