import json
import os
from flask import Flask, jsonify, request, send_from_directory
from flask_cors import CORS

app = Flask(__name__)

CORS(app)

# todo: make relative
DATA_FILE = '/Users/ryan/Github/Hackclub-projects/complete_database.json'


with open(DATA_FILE, 'r', encoding='utf-8') as f:
   all_projects = json.load(f)

@app.route("/")
def home():
    return send_from_directory(os.path.dirname(__file__), "index.html")

# blazingly slow
@app.route('/query', methods=['GET'])
def get_projects():

    search_query = request.args.get('q', '').lower()

    if not search_query:
        return jsonify(all_projects[:12])
    
    filtered_projects = [
        project for project in all_projects[:12] if search_query in repr(project)
    ]
    return jsonify(filtered_projects)
    
if __name__ == '__main__':
    app.run(debug=True, port=8022)