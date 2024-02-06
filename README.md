# KOSPes
Discord bot that notifies you about new term offers from KOS.

## Usage
1) Create `.env` file like this:
```
DISCORD=...
CLIENT_ID=...
CLIENT_SECRET=...
```
with `DISCORD` being the Discord token for your bot,
`CLIENT_ID` and `CLIENT_SECRET` being the two values needed for
obtaining the `access_token` to the Sirius API.

2) Create `config.toml` (TODO!)

3) Compile (`cargo build --release`) and run (`./target/release/kospes`) the bot 
