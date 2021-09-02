#!/usr/bin/env python3

# Python 3.x HTTP server for development purposes
# Client-side cache disabled
# Serving ./static folder
# Move this script to the project root to use it

# TESTING PURPOSES ONLY.
# DO NOT USE THIS SCRIPT IN PRODUCTION.

import http.server
import os

web_dir = os.path.join(os.path.dirname(__file__), 'static')
os.chdir(web_dir)

class MyHTTPRequestHandler(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_my_headers()
        http.server.SimpleHTTPRequestHandler.end_headers(self)

    def send_my_headers(self):
        self.send_header("Cache-Control", "no-cache, no-store, must-revalidate")
        self.send_header("Pragma", "no-cache")
        self.send_header("Expires", "0")


if __name__ == '__main__':
    http.server.test(HandlerClass=MyHTTPRequestHandler)

