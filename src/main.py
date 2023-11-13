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

    # Send a notification to all the subscribed channels
    async def send_notification(self, course: str, event):
        for channel_id in self.config.channels:
            # Try to get channel object from cache
            channel: Optional[
                Union[
                    discord.abc.GuildChannel,
                    discord.Thread,
                    discord.abc.PrivateChannel
                ]
            ] = self.get_channel(channel_id)

            # If channel is not found in the cache, fetch it
            if channel is None:
                channel = await self.fetch_channel(channel_id)

            # Check if the channel_id corresponds to actual text channel
            if(type(channel) is discord.TextChannel):
                # Construct the embed to send
                embed = discord.Embed()
                embed.title = f"[{course}]"

                # Be sure to check if every field actually exists
                if("starts_at" in event):
                    start = datetime.strptime(
                            event["starts_at"],
                            '%Y-%m-%dT%H:%M:%S.%f%z')
                    embed.add_field(
                        name = "Od",
                        value = f"<t:{int(start.timestamp())}:F>")

                if("ends_at" in event):
                    end = datetime.strptime(
                            event["ends_at"],
                            '%Y-%m-%dT%H:%M:%S.%f%z')
                    embed.add_field(
                        name = "Do",
                        value = f"<t:{int(end.timestamp())}:F>")

                if("capacity" in event):
                    embed.add_field(
                        name = "Kapacita",
                        value = event["capacity"],
                        inline = False)
                if("occupied" in event):
                    embed.add_field(
                        name = "Obsazenost",
                        value = event["occupied"],
                        inline = True)

                if("links" in event):
                    if("teachers" in event["links"]):
                        embed.set_footer(text = ", ".join(event["links"]["teachers"]))
                    if("room" in event["links"]):
                        embed.add_field(
                            name = "Místnost",
                            value = event["links"]["room"],
                            inline = False)

                if("note" in event):
                    if("cs" in event["note"]):
                        embed.add_field(
                            name = "Poznámka",
                            value = event["note"]["cs"],
                            inline = False)

                embed.colour = discord.Colour.blue()

                await channel.send(embed=embed)

    @tasks.loop(hours=2)
    async def update(self):
        # Fetch the newest events for each rouse
        for course in self.config.courses:
            response = await self.sirius.course_events(
                course,
                event_type = EventType.Assessment,
                start = datetime.now()
            )

            # For every event, check if we've seen it already
            if(response is not None and "events" in response):
                for event in response["events"]:
                    if "id" in event:
                        # If we haven't seen the event, save its ID
                        # and send out notifications to the subscribed channels
                        if event["id"] not in self.config.seen_events:
                            self.config.seen_events.append(event["id"])
                            await self.send_notification(course, event)

        self.config.save()

    async def on_ready(self):
        print(f"Logged in as {self.user}")
        self.update.start()
        print("Update loop started!")

if __name__ == "__main__":
    # Load .env file
    load_dotenv()

    # Fetch secrets from the environment or .env file
    token_result: Optional[str] = getenv("DISCORD")
    client_id_result: Optional[str] = getenv("CLIENT_ID")
    client_secret_result: Optional[str] = getenv("CLIENT_SECRET")

    if (token_result is not None and
        client_id_result is not None and
        client_secret_result is not None
    ):
        # We have to specify intents to declare
        # what the bot needs access to
        intents = discord.Intents.default()
        client = KOSPes(intents=intents)

        # Setup internal variables
        # Probably should be moved into __init__
        client.config = Config.load()
        client.sirius = SiriusAPI(client_id_result, client_secret_result)

        client.run(token_result)
    else:
        print("Failed to find some environment variables")
