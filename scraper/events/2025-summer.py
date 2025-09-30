# https://summer.hackclub.com
# https://journey.hackclub.com/gallery
import json
from typing import Annotated, List, Optional
from library import *
from browser import ScraperBrowser
from library import Text, Link, Time

class IndividualUpdateSOM:
    time:           Annotated[Time, './/div[@class="text-som-detail"]/span[1]']
    message:        Annotated[Text, './/div[@data-devlog-card-target="content"]']
    image:          Annotated[Optional[Link], './/img[contains(@class, "max-h-96")]']

class Summer2025:
    url:            Annotated[Url, '.']
    main_image:     Annotated[Link, '//div[contains(@class, "h-48")]//img']
    name:           Annotated[Text, '//h1']
    description:    Annotated[Text, '//div[contains(@class, "[p]:text-inherit")]/p']
    author:         Annotated[Text, '//span[contains(text(), "Created by")]/span/a']
    followers:      Annotated[int, '//button[@data-modal-type="follower"]/span']
    time:           Annotated[Time, '(//span[@class="text-som-dark"])[3]']
    readme:         Annotated[Optional[Link], '//button[@data-modal-type="readme"]']
    repo:           Annotated[Optional[Link], '//a[contains(., "Repository")]']
    demo:           Annotated[Optional[Link], '//a[contains(., "Demo")]']

    updates:        Annotated[List[IndividualUpdateSOM], '//div[@data-controller="modal devlog-card"]']


def run():
    return Summer2025