import discord

class KOSPes(discord.Client):
    async def on_ready(self):
        print(f"Logged in as {self.user}")

if __name__ == "__main__":
    intents = discord.Intents.default()
    client = KOSPes(intents=intents)
    client.run("token")
