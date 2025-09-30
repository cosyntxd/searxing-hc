# searxing-hc
A search engine that indexes all publically submitted hackclub projects. After timed events end, then it becomes difficult and sometimes impossible to view projects submitted. It saves an archive of the data so people can later view what the event was like. This can help the fraud team check for stolen projects or help users get inspired with new ideas.

# running
scraping: `python3 scraper/main.py`

hosting: `cd backend && cargo r --release`

# project structure
backend - the actual search and ranking engine

scraper - spawns an automated chrome instance to scrape events

scripts - deploying and updating constants

website - the public facing html frontend

