# https://summer.hackclub.com
# https://journey.hackclub.com/gallery
import json
from typing import Annotated, List, Optional
from library import *
from browser import ScraperBrowser
from library import Text, Link, Time

class IndividualUpdateSOM:
    time:           Annotated[Time, './/div[@class="text-[#B89576]"]/span[1]']
    message:        Annotated[Text, './/div[@class="prose max-w-[32em] text-[#4a2d24] mb-2 sm:mb-3 text-base sm:text-lg 2xl:text-xl break-words overflow-wrap-anywhere"]']
    image:          Annotated[Optional[Link], './/img[@class="w-full object-contain cursor-pointer hover:opacity-90 transition-opacity rounded-lg max-h-96"]']

class Summer2025:
    url:            Annotated[Url, '.']
    main_image:     Annotated[Link, '//img[@class="object-contain w-full h-full max-h-full max-w-full"]']
    name:           Annotated[Text, '//h1[@class="text-2xl md:text-3xl 2xl:text-4xl text-black md:flex-grow"]']
    description:    Annotated[Text, '//div[@class="text-black mb-3 md:mb-4 text-base md:text-lg 2xl:text-xl [p]:text-inherit"]/p']
    author:         Annotated[Text, '//div[@class="flex items-center space-x-2 mb-3 md:mb-4 text-sm md:text-base 2xl:text-lg text-gray-600"]/a/span/span']
    followers:      Annotated[int, '(//span[@class="text-gray-800"])[1]']
    time:           Annotated[Time, '(//span[@class="text-gray-800"])[3]']
    readme:         Annotated[Optional[Link], '//a[@class="px-3 md:px-4 py-3 md:py-2 bg-saddle-taupe hover:scale-[1.05] text-white transition-transform duration-300 flex-1 text-center btn-pixel text-base md:text-lg 2xl:text-2xl"]']
    repo:           Annotated[Optional[Link], '//a[@class="px-3 md:px-4 py-3 md:py-2 bg-nice-blue hover:scale-[1.05] text-white transition-transform duration-300 flex-1 text-center btn-pixel text-base md:text-lg 2xl:text-2xl"]']
    demo:           Annotated[Optional[Link], '//a[@class="px-3 md:px-4 py-3 md:py-2 bg-forest hover:scale-[1.05] text-white transition-transform duration-300 flex-1 text-center btn-pixel text-base md:text-lg 2xl:text-2xl"]']

    updates:        Annotated[List[IndividualUpdateSOM], '//div[@class="card-content"]']



def run():
    return Summer2025