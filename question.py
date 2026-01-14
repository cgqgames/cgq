class Question:
    def __init__(self, question_text="", options=None, correct_index=0):
        self.question_text = question_text
        self.options = options or ["", "", "", ""]
        self.correct_index = correct_index

    def to_dict(self):
        return {
            'question_text': self.question_text,
            'options': self.options.copy(),
            'correct_index': self.correct_index
        }

    @classmethod
    def from_dict(cls, data):
        # Check for required fields with default values
        question_text = data.get('question_text', '')
        options = data.get('options', ["", "", "", ""])
        correct_index = data.get('correct_index', 0)

        # Ensure we have exactly 4 options and they are strings
        if not isinstance(options, list):
            options = ["", "", "", ""]

        # Make sure we have 4 options, pad with empty strings if needed
        while len(options) < 4:
            options.append("")
        if len(options) > 4:
            options = options[:4]

        # Ensure correct_index is within bounds
        if correct_index < 0 or correct_index >= 4:
            correct_index = 0

        return cls(
            question_text=question_text,
            options=options,
            correct_index=correct_index
        )

    def is_correct(self, selected_index):
        return selected_index == self.correct_index
