# KOSPes
Discord bot that notifies you about new term offers from KOS.

## Requirements
- Python 3.11+

Python modules:

- `discord`
- `dotenv`
- `aiohttp`

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

2) Run the bot (`main.py`)

3) Edit the `config.json` to include the desired channels and courses
```
{
    "channels": [
        111111111111111111,
        222222222222222222
    ],
    "courses": [
        "BI-ULI",
        "BI-LA1.21",
        "BI-DML.21"
    ],
    "seen_events": []
}
```
