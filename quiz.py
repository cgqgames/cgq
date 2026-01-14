import json
from question import Question

class Quiz:
    def __init__(self, title="", description=""):
        self.title = title
        self.description = description
        self.questions = []

    def add_question(self, question):
        self.questions.append(question)

    def remove_question(self, index):
        if 0 <= index < len(self.questions):
            self.questions.pop(index)

    def to_dict(self):
        return {
            'title': self.title,
            'description': self.description,
            'questions': [q.to_dict() for q in self.questions]
        }

    @classmethod
    def from_dict(cls, data):
        # Check for required fields with default values
        title = data.get('title', 'Untitled Quiz')
        description = data.get('description', '')

        quiz = cls(title=title, description=description)

        # Load questions if they exist
        questions_data = data.get('questions', [])
        quiz.questions = []
        for q_data in questions_data:
            try:
                question = Question.from_dict(q_data)
                quiz.questions.append(question)
            except Exception as e:
                print(f"Warning: Failed to load question: {e}")
                continue

        return quiz

    def save_to_file(self, filename):
        with open(filename, 'w', encoding='utf-8') as f:
            json.dump(self.to_dict(), f, indent=2, ensure_ascii=False)

    @classmethod
    def load_from_file(cls, filename):
        try:
            with open(filename, 'r', encoding='utf-8') as f:
                data = json.load(f)

            # Validate basic structure
            if not isinstance(data, dict):
                raise Exception("Invalid quiz file format: expected JSON object")

            return cls.from_dict(data)

        except json.JSONDecodeError as e:
            raise Exception(f"Invalid JSON format: {str(e)}")
        except Exception as e:
            raise Exception(f"Error loading quiz file: {str(e)}")
