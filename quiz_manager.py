import os
import json
from quiz import Quiz

class QuizManager:
    def __init__(self):
        self.quizzes = []
        self.current_quiz_index = -1

    def add_quiz(self, quiz):
        self.quizzes.append(quiz)

    def remove_quiz(self, index):
        if 0 <= index < len(self.quizzes):
            self.quizzes.pop(index)

    def get_quiz(self, index):
        if 0 <= index < len(self.quizzes):
            return self.quizzes[index]
        return None

    def save_all(self, directory="quizzes"):
        if not os.path.exists(directory):
            os.makedirs(directory)

        data = {
            'quizzes': [quiz.to_dict() for quiz in self.quizzes]
        }

        with open(os.path.join(directory, 'quizzes.json'), 'w', encoding='utf-8') as f:
            json.dump(data, f, indent=2, ensure_ascii=False)

    def load_all(self, directory="quizzes"):
        filepath = os.path.join(directory, 'quizzes.json')
        if os.path.exists(filepath):
            try:
                with open(filepath, 'r', encoding='utf-8') as f:
                    data = json.load(f)

                # Clear current quizzes and load from file
                self.quizzes = []
                for quiz_data in data.get('quizzes', []):
                    quiz = Quiz.from_dict(quiz_data)
                    self.quizzes.append(quiz)

            except json.JSONDecodeError as e:
                print(f"Warning: Invalid JSON in {filepath}: {e}")
                self.quizzes = []
            except Exception as e:
                print(f"Warning: Error loading quizzes: {e}")
                self.quizzes = []
