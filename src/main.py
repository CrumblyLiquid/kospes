import discord
from discord.ext import tasks

from typing import Optional, Union

from datetime import datetime
from dotenv import load_dotenv
from os import getenv

from sirius import SiriusAPI, EventType
from config import Config

class KOSPes(discord.Client):
    config: Config
    sirius: SiriusAPI

    async def send_notification(self, course: str, event):
        for channel_id in self.config.channels:
            channel: Optional[Union[discord.abc.GuildChannel, discord.Thread, discord.abc.PrivateChannel]] = self.get_channel(channel_id)
            if channel is None:
                channel = await self.fetch_channel(channel_id)
            if(type(channel) is discord.TextChannel):
                embed = discord.Embed()
                embed.title = f"[{course}]"

                if("starts_at" in event):
                    start = datetime.strptime(event["starts_at"], '%Y-%m-%dT%H:%M:%S.%f%z')
                    start_unix = int(start.timestamp())
                    embed.add_field(name = "Od", value = f"<t:{start_unix}:F>")

                if("ends_at" in event):
                    end = datetime.strptime(event["ends_at"], '%Y-%m-%dT%H:%M:%S.%f%z')
                    end_unix = int(end.timestamp())
                    embed.add_field(name = "Do", value = f"<t:{end_unix}:F>")

                if("capacity" in event):
                    embed.add_field(name = "Kapacita", value = event["capacity"], inline = False)
                if("occupied" in event):
                    embed.add_field(name = "Obsazenost", value = event["occupied"], inline = True)

                if("links" in event):
                    if("teachers" in event["links"]):
                        embed.set_footer(text = ", ".join(event["links"]["teachers"]))
                    if("room" in event["links"]):
                        embed.add_field(name = "Místnost", value = event["links"]["room"], inline = False)

                if("note" in event):
                    if("cs" in event["note"]):
                        embed.add_field(name = "Poznámka", value = event["note"]["cs"], inline = False)

                embed.colour = discord.Colour.blue()

                await channel.send(embed=embed)

    @tasks.loop(hours=2)
    async def update(self):
        for course in self.config.courses:
            info = await self.sirius.course_events(
                course,
                event_type = EventType.Assessment,
                start = datetime.now()
            )

            if(info is not None and "events" in info):
                for event in info["events"]:
                    if "id" in event:
                        if event["id"] not in self.config.seen_events:
                            self.config.seen_events.append(event["id"])
                            await self.send_notification(course, event)

        self.config.save()

    async def on_ready(self):
        print(f"Logged in as {self.user}")
        self.update.start()
        print("Update loop started!")

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
