# How to set up an InvArch Collator

Make sure you are logged in as a user with root privileges, otherwise:

``sudo su -``

Create a new service user to run your collator service:
`sudo useradd --no-create-home --shell /usr/sbin/nologin tinkernet-collator`

Install some required stuff:

``curl https://sh.rustup.rs -sSf | sh``
(choose option 1 - Proceed with installation (default))
You may need to restart your system before the next steps.
```
apt install cmake git clang libclang-dev
```
Type Y to proceed.

Clone the repo:

 ``git clone git@github.com:InvArch/InvArch-Node.git``

Type the following command, and make sure you can see "InvArch-Node", if not then you likely did something wrong.

  ``ls -la``

Enter the repo and check out the most recent tagged release of code (https://github.com/InvArch/InvArch-Node/releases)

```
cd InvArch-Node
git checkout $(git describe --tags $(git rev-list --tags --max-count=1))
```

You need to compile the code, this will take quite a while depending on your system (30+ minutes is normal):
 ``cargo build --release --features tinkernet``

Move the node executable to `usr/local/bin`, make it executable, and change ownership to our `tinkernet-collator` service user:
```
sudo mv ~/InvArch-Node/target/release/invarch-collator /usr/local/bin/tinkernet-collator
sudo chmod +x /usr/local/bin/tinkernet-collator
sudo chown tinkernet-collator:tinkernet-collator /usr/local/bin/tinkernet-collator
```

Create the base-path folder, copy the tinker-raw "chainspec" into it, and give it the necessary permissions & ownership:
```
sudo mkdir /var/lib/tinkernet
sudo cp ~/InvArch-Node/res/kusama/tinker-raw.json /var/lib/tinkernet/tinker-raw.json
sudo chown -R tinkernet-collator:tinkernet-collator /var/lib/tinkernet
```

Create a systemd service file to run your collator (and automatically restart it):

`sudo nano /etc/systemd/system/tinkernet-collator.service`

Within that file, paste in the following:
```
[Unit]
Description=Tinkernet (InvArch) Collator
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=tinkernet-collator
Group=tinkernet-collator
ExecStart=/usr/local/bin/tinkernet-collator \
  --base-path /var/lib/tinkernet \
  --collator \
  --force-authoring \
  --name "YOUR-COLLATOR-NAME-HERE" \
  --chain /var/lib/tinkernet/tinker-raw.json \
  --listen-addr "/ip4/0.0.0.0/tcp/30333/ws" \
  --telemetry-url "wss://telemetry.polkadot.io/submit 0" \
  -- \
  --execution wasm \
  --chain kusama \
  --database=RocksDb \
  --unsafe-pruning \
  --pruning=1000 \
  --port 30343

Restart=always
RestartSec=120
[Install]
WantedBy=multi-user.target
```

Then ctrl + s then ctrl + x to save & exit that file.

Let's start the collator:

`sudo systemctl daemon-reload && sudo systemctl enable tinkernet-collator && sudo systemctl start tinkernet-collator.service`

Now, let's check that the chain is running

``sudo systemctl status tinkernet-collator.service``

If the service indicates it's "running" and you see no errors, you should be ok. If not, you can debug using one of the following:
`sudo journalctl -fu tinkernet-collator`
`sudo systemctl status --full --lines=100 tinkernet-collator`

Check if your node appears here (from your browser):

``https://telemetry.polkadot.io/#list/0x19a3733beb9cb8a970a308d835599e9005e02dc007a35440e461a451466776f8``

Syncing the Kusama relaychain will take a long time, depending on your download speed (it needs to download something like 130 gb via P2P). If you'd like to accelerate that process you can download a snapshot of the Kusama relaychain to start with:

``sudo systemctl stop tinkernet-collator.service``

``ls /var/lib/tinkernet``

You should see "chains" and "polkadot" directories.

``sudo apt install curl lz4 tar``

Enter y to continue if prompted.

``sudo rm -rf /var/lib/tinkernet/polkadot/chains/ksmcc3/*``

``sudo curl -o - -L https://ksm-rocksdb.polkashots.io/snapshot | sudo lz4 -c -d - | sudo tar -x -C /var/lib/tinkernet/polkadot/chains/ksmcc3``

Once that's downloaded, we need to make sure you add your account to the collator, I would strongly reccomend making a new account for that... go to polkadot.js.org, and make a new account but save the "raw seed", and not the mnemonic.

``sudo /usr/local/bin/tinkernet-collator key insert --base-path /var/lib/tinkernet --chain /var/lib/tinkernet/tinker-raw.json --scheme Sr25519 --suri "your_private_key(RAW SEED)_here" --password-interactive --key-type aura``

Note: see more here if you want to double check the above https://docs.substrate.io/tutorials/get-started/trusted-network/#add-keys-to-the-keystore

Now we can start the the service again:

``sudo systemctl start tinkernet-collator.service``

Now we need to rotate keys and set our keys on chain.

Ensure the collator is running, or this step won't work:

``curl -H "Content-Type: application/json" -d '{"id":1, "jsonrpc":"2.0", "method": "author_rotateKeys", "params":[]}' http://127.0.0.1:9933/``

You will be greeted with an output that looks like:

``{"jsonrpc":"2.0","result":"0xprivate_key_will_be_here0","id":1}``

"result":"**0x_private_key_will_be_here0**" is what we are interested in.

You need to make sure that you have a Polkadot/Substrate account set up, here's some videos in case you don't know how to do that:

1. Polkadot JS Video  https://www.youtube.com/watch?v=dG0DP9vayPY    https://www.youtube.com/watch?v=BpTQBAyFvEk

2. Talisman Video   https://docs.talisman.xyz/talisman/talisman-initiation/setup-a-talisman-wallet  

Now that you have made an account using one of those extensions, head on over to the InvArch Tinkernet section of Polkadot JS: 

https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Ftinker.invarch.network#/extrinsics

Ensure that you are in the Developer tab (the top header), and navigate to the extrinsics section in the drop down.


In the "using the selected account field" select the account you just made for the collator.<br/>
In the "submit the following extrinsic field" select "session".<br/>
In the next field (to the right), select "setKeys(keys, proof)".<br/>
In the "keys:" field, paste in your **0x_private_key_will_be_here0** from your node.<br/>
In the "proof" field, type in " 0 ".

Submit the transaction.

Congratulations, you should now be onboarded as a collator.
