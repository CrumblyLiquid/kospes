from enum import Enum
from typing import Optional
from dotenv import load_dotenv
from os import getenv
import datetime
import time
import aiohttp
import asyncio

class EventType(Enum):
    Assessment = "assessment"
    CourseEvent = "course_event"
    Exam = "exam"
    Laboratory = "laboratory"
    Lecture = "lecture"
    Tutorial = "tutorial"

class SiriusAPI:
    url: str = "https://sirius.fit.cvut.cz/api/v1"
    client_id: str
    client_secret: str
    access_token: str | None = None
    expires_in: int = 0

    def __init__(self, client_id: str, client_secret: str):
        self.client_id = client_id
        self.client_secret = client_secret

    async def get_access_token(self):
        if self.access_token is None or int(time.time()) >= self.expires_in:
            params = {
                "grant_type": "client_credentials",
                "client_id": self.client_id,
                "client_secret": self.client_secret,
                "scope": "cvut:sirius:personal:read"
            }

            async with aiohttp.ClientSession() as session:
                url: str = "https://auth.fit.cvut.cz/oauth/oauth/token"
                async with session.post(url, params=params) as response:
                    if(response.status == 200):
                        content = await response.json()
                        self.access_token = content["access_token"]
                        self.expires_in = int(time.time()) + int(content["expires_in"])

        return self.access_token

    async def course_events(self,
                            course: str,
                            limit: Optional[int],
                            offset: Optional[int],
                            include: Optional[str],
                            event_type: Optional[EventType],
                            deleted: Optional[bool],
                            start: Optional[datetime.datetime],
                            end: Optional[datetime.datetime],
                            with_original_date: Optional[bool]
    ) -> Optional[dict]:
        params = {
            "access_token": await self.get_access_token(),
        }
        
        if limit is not None:
            params["limit"] = str(limit)
        if offset is not None:
            params["offset"] = str(offset)
        if include is not None:
            params["include"] = include
        if event_type is not None:
            params["event_type"] = str(event_type.value)
        if deleted is not None:
            params["deleted"] = str(deleted).lower()
        if start is not None:
            params["from"] = start.strftime("%Y-%m-%dT%H:%M:%S.%f%z")
        if end is not None:
            params["to"] = end.strftime("%Y-%m-%dT%H:%M:%S.%f%z")
        if with_original_date is not None:
            params["with_original_date"] = str(with_original_date).lower()

        async with aiohttp.ClientSession() as session:
            url: str = f"{self.url}/courses/{course}/events"
            async with session.get(url, params=params) as response:
                if(response.status == 200):
                    return await response.json()
                else:
                    print(await response.text())

async def main():
    load_dotenv()

    client_id_result: str | None = getenv("CLIENT_ID")
    client_secret_result: str | None = getenv("CLIENT_SECRET")

    if (client_id_result is not None and
        client_secret_result is not None
    ):
        api = SiriusAPI(
            client_id_result,
            client_secret_result)
        print(await api.course_events("BI-LA1.21"))

if __name__ == "__main__":
    asyncio.run(main())
