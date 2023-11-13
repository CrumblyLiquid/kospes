import discord
from discord.ext import tasks

from dotenv import load_dotenv
from os import getenv

from sirius import SiriusAPI
from config import Config

class KOSPes(discord.Client):
    config: Config
    sirius: SiriusAPI

    async def on_ready(self):
        print(f"Logged in as {self.user}")
        print("Starting loops")

if __name__ == "__main__":
    load_dotenv()

    intents = discord.Intents.default()
    client = KOSPes(intents=intents)

    token_result: str | None = getenv("DISCORD")
    client_id_result: str | None = getenv("CLIENT_ID")
    client_secret_result: str | None = getenv("CLIENT_SECRET")

    if (token_result is not None and
        client_id_result is not None and
        client_secret_result is not None
    ):
        client.config = Config.load()
        client.sirius = SiriusAPI(client_id_result, client_secret_result)
        client.run(token_result)
    else:
        print("Failed to find some environment variables")
