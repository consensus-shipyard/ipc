import { task, types } from 'hardhat/config';
import { HardhatRuntimeEnvironment, TaskArguments } from 'hardhat/types';
import * as fs from 'fs';
import { Wallet } from 'ethers';

task('generate-wallets', 'Generate a specified number of wallet addresses')
  .addParam('n', 'Number of wallets to generate', undefined, types.int)
  .setAction(async (taskArgs: TaskArguments, hre: HardhatRuntimeEnvironment) => {
    const n = taskArgs.n;

    if (isNaN(n) || n <= 0) {
      console.error('❌ Please provide a valid number greater than 0.');
      process.exit(1);
    }

    const wallets: { address: string; privateKey: string }[] = [];

    for (let i = 0; i < n; i++) {
      const wallet = hre.ethers.Wallet.createRandom();
      wallets.push({
        address: wallet.address,
        privateKey: wallet.privateKey,
      });
    }

    console.log('Generated Wallets:');
    console.table(wallets);

    // Save to JSON file
    fs.writeFileSync('generated-wallets.json', JSON.stringify(wallets, null, 2));
    console.log(`✅ Successfully generated ${n} wallets and saved to 'generated-wallets.json'`);
  });


  
  task('spam-from-wallets', 'Spam the network using wallets from a JSON file')
    .addParam('file', 'Path to the JSON file with wallet keys', undefined, types.string)
    .addParam('x', 'Multiplier for the gas fees', undefined, types.float)
    .addParam('count', 'Number of transactions per wallet', undefined, types.int)
    .addParam('gasLimit', 'Gas limit for each transaction', undefined, types.int)
    .setAction(async (taskArgs: TaskArguments, hre: HardhatRuntimeEnvironment) => {
      const { file, x, count, gasLimit } = taskArgs;
  
      if (x <= 0 || count <= 0 || gasLimit <= 0) {
        console.error('❌ All parameters must be greater than 0.');
        process.exit(1);
      }
  
      // Load wallets from file
      if (!fs.existsSync(file)) {
        console.error(`❌ File not found: ${file}`);
        process.exit(1);
      }
  
      const walletsData = JSON.parse(fs.readFileSync(file, 'utf8')) as { address: string; privateKey: string }[];
      if (walletsData.length === 0) {
        console.error('❌ No wallets found in the file.');
        process.exit(1);
      }
  
      console.log(`📂 Loaded ${walletsData.length} wallets from '${file}'`);
  
      // Get current gas params from the latest block
      const provider = hre.ethers.provider;
      const latestBlock = await provider.getBlock('latest');
  
      if (!latestBlock || !latestBlock.baseFeePerGas) {
        console.error('❌ Unable to retrieve current gas params.');
        process.exit(1);
      }
  
      const currentBaseFee = latestBlock.baseFeePerGas;
      const currentPriorityFee = hre.ethers.utils.parseUnits('1', 'gwei');
  
      console.log(`🔎 Current Base Fee Per Gas: ${hre.ethers.utils.formatUnits(currentBaseFee, 'gwei')} gwei`);
      console.log(`🔎 Current Priority Fee Per Gas: ${hre.ethers.utils.formatUnits(currentPriorityFee, 'gwei')} gwei`);
  
      const newMaxFeePerGas = currentBaseFee.mul(Math.floor(x));
      const newPriorityFeePerGas = currentPriorityFee.mul(Math.floor(x));
  
      console.log(`🚀 New Max Fee Per Gas: ${hre.ethers.utils.formatUnits(newMaxFeePerGas, 'gwei')} gwei`);
      console.log(`🚀 New Priority Fee Per Gas: ${hre.ethers.utils.formatUnits(newPriorityFeePerGas, 'gwei')} gwei`);
  
      // Define an async function to send transactions from a wallet
      const spamFromWallet = async (walletData: { address: string; privateKey: string }) => {
        const wallet = new Wallet(walletData.privateKey, provider);
  
        console.log(`📤 Spamming from wallet: ${wallet.address}`);
  
        const txPromises = Array.from({ length: count }).map(async (_, i) => {
          try {
            const tx = await wallet.sendTransaction({
              to: wallet.address, // Send to self to burn gas
              value: hre.ethers.utils.parseEther('0'), // No value transfer
              gasLimit,
              maxFeePerGas: newMaxFeePerGas,
              maxPriorityFeePerGas: newPriorityFeePerGas,
            });
  
            console.log(`✅ Transaction ${i + 1} from ${wallet.address} sent: ${tx.hash}`);
            await tx.wait();
          } catch (error) {
            console.error(`❌ Transaction ${i + 1} from ${wallet.address} failed: ${error.message}`);
          }
        });
  
        await Promise.all(txPromises);
      };
  
      // Use Promise.all to spam from all wallets in parallel
      await Promise.all(walletsData.map(spamFromWallet));
  
      console.log(`✅ All spam transactions completed.`);
    });

task('fund-wallets', 'Fund wallets from a JSON file with Ether using nonce management')
  .addParam('file', 'Path to the JSON file with wallet keys', undefined, types.string)
  .addParam('amount', 'Amount of Ether to send to each wallet', undefined, types.float)
  .setAction(async (taskArgs: TaskArguments, hre: HardhatRuntimeEnvironment) => {
    const { file, amount } = taskArgs;

    if (amount <= 0) {
      console.error('❌ Amount must be greater than 0.');
      process.exit(1);
    }

    // Load wallet data from JSON file
    if (!fs.existsSync(file)) {
      console.error(`❌ File not found: ${file}`);
      process.exit(1);
    }

    const walletsData = JSON.parse(fs.readFileSync(file, 'utf8')) as { address: string; privateKey: string }[];
    if (walletsData.length === 0) {
      console.error('❌ No wallets found in the file.');
      process.exit(1);
    }

    console.log(`📂 Loaded ${walletsData.length} wallets from '${file}'`);

    // Get the first signer (usually the deployer)
    const [signer] = await hre.ethers.getSigners();
    const signerBalance = await hre.ethers.provider.getBalance(signer.address);

    console.log(`👤 Funding wallets using signer: ${signer.address}`);
    console.log(`💰 Signer Balance: ${hre.ethers.utils.formatEther(signerBalance)} ETH`);

    if (signerBalance.lt(hre.ethers.utils.parseEther((amount * walletsData.length).toString()))) {
      console.error(`❌ Not enough balance to fund all wallets.`);
      process.exit(1);
    }

    const fundingAmount = hre.ethers.utils.parseEther(amount.toString());

    // 🔥 Get the current nonce for the signer
    let currentNonce = await hre.ethers.provider.getTransactionCount(signer.address);

    console.log(`📌 Starting nonce: ${currentNonce}`);

    // Function to send transactions with manual nonce management
    const fundWallet = async (address: string, nonce: number) => {
      try {
        const tx = await signer.sendTransaction({
          to: address,
          value: fundingAmount,
          nonce // ✅ Manually set the nonce
          // 🔥 No need to set gas params — let Hardhat handle it!
        });

        console.log(`✅ Sent ${amount} ETH to ${address} | Tx: ${tx.hash} | Nonce: ${nonce}`);
      } catch (error) {
        console.error(`❌ Failed to send to ${address} | Nonce: ${nonce} | Error: ${error.message}`);
      }
    };

    // 🔥 Send all transactions concurrently using Promise.all
    await Promise.all(
      walletsData.map((wallet, index) => {
        // Manually set nonce for each transaction
        const nonce = currentNonce + index;
        return fundWallet(wallet.address, nonce);
      })
    );

    console.log(`✅ All wallets funded successfully.`);
  });

  import { task, types } from 'hardhat/config';
import { HardhatRuntimeEnvironment, TaskArguments } from 'hardhat/types';
import * as fs from 'fs';
import { Wallet } from 'ethers';

task('send-tx-with-fee', 'Send transactions from wallets with specified maxFeePerGas')
  .addParam('file', 'Path to the JSON file with wallet keys', undefined, types.string)
  .addParam('maxFee', 'Max fee per gas in gwei', undefined, types.float)
  .addParam('value', 'Ether value to send in each transaction', undefined, types.float)
  .setAction(async (taskArgs: TaskArguments, hre: HardhatRuntimeEnvironment) => {
    const { file, maxFee, value } = taskArgs;

    if (maxFee <= 0 || value <= 0) {
      console.error('❌ maxFee and value must be greater than 0.');
      process.exit(1);
    }

    // Load wallet data from JSON file
    if (!fs.existsSync(file)) {
      console.error(`❌ File not found: ${file}`);
      process.exit(1);
    }

    const walletsData = JSON.parse(fs.readFileSync(file, 'utf8')) as { address: string; privateKey: string }[];
    if (walletsData.length === 0) {
      console.error('❌ No wallets found in the file.');
      process.exit(1);
    }

    console.log(`📂 Loaded ${walletsData.length} wallets from '${file}'`);

    // Convert maxFee to wei
    const maxFeePerGas = hre.ethers.utils.parseUnits(maxFee.toString(), 'gwei');
    const sendValue = hre.ethers.utils.parseEther(value.toString());

    console.log(`🚀 maxFeePerGas: ${hre.ethers.utils.formatUnits(maxFeePerGas, 'gwei')} gwei`);
    console.log(`🚀 Value per txn: ${value} ETH`);

    // 🔥 Get the first signer (usually the deployer)
    const [signer] = await hre.ethers.getSigners();

    console.log(`👤 Sending from signer: ${signer.address}`);

    let currentNonce = await hre.ethers.provider.getTransactionCount(signer.address);
    console.log(`📌 Starting nonce: ${currentNonce}`);

    // Function to send transaction with manual nonce
    const sendTx = async (wallet: { address: string; privateKey: string }, nonce: number) => {
      try {
        const walletSigner = new Wallet(wallet.privateKey, hre.ethers.provider);

        const tx = await walletSigner.sendTransaction({
          to: signer.address, // Send to deployer address (or any address you prefer)
          value: sendValue,
          nonce, // ✅ Manually set nonce
          maxFeePerGas,
          maxPriorityFeePerGas: hre.ethers.utils.parseUnits('1', 'gwei'), // Keep priority fee low
        });

        console.log(`✅ Tx from ${wallet.address} sent: ${tx.hash} | Nonce: ${nonce}`);
      } catch (error) {
        console.error(`❌ Failed to send from ${wallet.address} | Nonce: ${nonce} | Error: ${error.message}`);
      }
    };

    // 🔥 Send all transactions concurrently using Promise.all
    await Promise.all(
      walletsData.map((wallet, index) => {
        const nonce = currentNonce + index;
        return sendTx(wallet, nonce);
      })
    );

    console.log(`✅ All transactions sent successfully.`);
  });

  task('watch-tx-and-gas', 'Continuously check block gas data and transaction status')
    .addOptionalParam('hash', 'Transaction hash to monitor', undefined, types.string)
    .setAction(async (taskArgs, hre) => {
      const { hash } = taskArgs;
      const provider = hre.ethers.provider;
  
      let txConfirmed = false;
  
      console.log('Watching gas data and transaction status... (Press Ctrl + C to stop)');
  
      const pollData = async () => {
        try {
          const latestBlock = await provider.getBlock('latest');
          const feeData = await provider.getFeeData();
  
          const baseFeePerGas = latestBlock.baseFeePerGas
            ? `${hre.ethers.utils.formatUnits(latestBlock.baseFeePerGas, 'gwei')} gwei`
            : 'Not available';
  
          const gasLimit = latestBlock.gasLimit.toString();
  
          const maxPriorityFeePerGas = feeData.maxPriorityFeePerGas
            ? `${hre.ethers.utils.formatUnits(feeData.maxPriorityFeePerGas, 'gwei')} gwei`
            : 'Not available';
  
          const maxFeePerGas = feeData.maxFeePerGas
            ? `${hre.ethers.utils.formatUnits(feeData.maxFeePerGas, 'gwei')} gwei`
            : 'Not available';
  
          console.log(`Block: ${latestBlock.number}`);
          console.log(`Base Fee: ${baseFeePerGas}`);
          console.log(`Gas Limit: ${gasLimit}`);
          console.log(`Max Priority Fee: ${maxPriorityFeePerGas}`);
          console.log(`Max Fee: ${maxFeePerGas}`);
  
          if (hash) {
            const receipt = await provider.getTransactionReceipt(hash);
  
            if (receipt) {
              txConfirmed = true;
  
              console.log(`Transaction Hash: ${receipt.transactionHash}`);
              console.log(`Block Number: ${receipt.blockNumber}`);
              console.log(`From: ${receipt.from}`);
              console.log(`To: ${receipt.to || 'Contract Creation'}`);
              console.log(`Gas Used: ${receipt.gasUsed.toString()}`);
              console.log(`Effective Gas Price: ${hre.ethers.utils.formatUnits(receipt.effectiveGasPrice, 'gwei')} gwei`);
  
              if (receipt.status === 1) {
                console.log('Status: SUCCESS');
              } else {
                console.log('Status: FAILED');
              }
            } else {
              console.log('Transaction Status: PENDING');
            }
          }
  
          console.log('--------------------------------');
        } catch (error) {
          console.error(`Error: ${error.message}`);
        }
      };
  
      // Run the loop every 3 seconds
      const interval = setInterval(async () => {
        await pollData();
  
        if (txConfirmed) {
          clearInterval(interval);
          console.log(`Transaction confirmed. Stopping monitoring.`);
          process.exit(0);
        }
      }, 3000);
  
      // Run it once immediately
      await pollData();
  
      // Keep the process running (until Ctrl + C or process.exit)
      await new Promise(() => {});
    });
  