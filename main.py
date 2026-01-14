import sys
import os
from PyQt5.QtWidgets import (QApplication, QMainWindow, QWidget, QVBoxLayout,
                             QHBoxLayout, QPushButton, QListWidget, QListWidgetItem,
                             QLabel, QLineEdit, QTextEdit, QRadioButton, QButtonGroup,
                             QMessageBox, QTabWidget, QGroupBox, QScrollArea, QDialog,
                             QDialogButtonBox, QSplitter, QInputDialog, QFileDialog, QFrame)
from PyQt5.QtCore import Qt
from PyQt5.QtGui import QFont

from quiz_manager import QuizManager
from quiz import Quiz
from question import Question

class QuestionEditorDialog(QDialog):
    def __init__(self, question=None, parent=None):
        super().__init__(parent)
        self.question = question or Question()
        self.init_ui()
        self.load_question_data()

    def init_ui(self):
        self.setWindowTitle("Edit Question")
        self.setModal(True)
        self.resize(500, 400)

        layout = QVBoxLayout()

        # Question text
        layout.addWidget(QLabel("Question:"))
        self.question_edit = QTextEdit()
        layout.addWidget(self.question_edit)

        # Options
        options_group = QGroupBox("Options (select correct answer)")
        options_layout = QVBoxLayout()

        self.option_edits = []
        self.option_radios = []
        self.button_group = QButtonGroup()

        for i in range(4):
            option_layout = QHBoxLayout()
            radio = QRadioButton()
            self.button_group.addButton(radio, i)
            self.option_radios.append(radio)

            option_edit = QLineEdit()
            self.option_edits.append(option_edit)

            option_layout.addWidget(radio)
            option_layout.addWidget(option_edit)
            options_layout.addLayout(option_layout)

        options_group.setLayout(options_layout)
        layout.addWidget(options_group)

        # Buttons
        button_layout = QHBoxLayout()
        ok_btn = QPushButton("OK")
        cancel_btn = QPushButton("Cancel")

        ok_btn.clicked.connect(self.accept)
        cancel_btn.clicked.connect(self.reject)

        button_layout.addWidget(ok_btn)
        button_layout.addWidget(cancel_btn)
        layout.addLayout(button_layout)

        self.setLayout(layout)

    def load_question_data(self):
        self.question_edit.setPlainText(self.question.question_text)
        for i in range(4):
            self.option_edits[i].setText(self.question.options[i])
        self.option_radios[self.question.correct_index].setChecked(True)

    def get_question(self):
        self.question.question_text = self.question_edit.toPlainText()
        for i in range(4):
            self.question.options[i] = self.option_edits[i].text()

        # Find correct answer
        for i in range(4):
            if self.option_radios[i].isChecked():
                self.question.correct_index = i
                break

        return self.question

class QuizDisplayDialog(QDialog):
    def __init__(self, quiz, parent=None):
        super().__init__(parent)
        self.quiz = quiz
        self.current_question_index = 0
        self.score = 0
        self.user_answers = []
        self.init_ui()
        self.load_question()

    def init_ui(self):
        self.setWindowTitle(f"Take Quiz: {self.quiz.title}")
        self.setGeometry(100, 100, 700, 500)

        layout = QVBoxLayout()

        # Header with quiz info
        header_label = QLabel(f"Quiz: {self.quiz.title}")
        header_label.setFont(QFont("Arial", 16, QFont.Bold))
        layout.addWidget(header_label)

        if self.quiz.description:
            desc_label = QLabel(self.quiz.description)
            desc_label.setWordWrap(True)
            layout.addWidget(desc_label)

        # Progress
        self.progress_label = QLabel()
        self.progress_label.setFont(QFont("Arial", 16))
        layout.addWidget(self.progress_label)

        # Question
        self.question_label = QLabel()
        self.question_label.setFont(QFont("Arial", 20, QFont.Bold))
        self.question_label.setWordWrap(True)
        self.question_label.setStyleSheet("background-color: #0f0f0f; padding: 10px; border-radius: 5px;")
        layout.addWidget(self.question_label)

        # Options
        self.option_group = QButtonGroup()
        self.option_buttons = []

        for i in range(4):
            option_btn = QRadioButton()
            option_btn.setStyleSheet("padding: 8px; margin: 2px;")
            option_btn.setFont(QFont("Arial", 19))
            self.option_buttons.append(option_btn)
            self.option_group.addButton(option_btn, i)
            layout.addWidget(option_btn)

        # Navigation buttons
        nav_layout = QHBoxLayout()

        self.prev_btn = QPushButton("Previous")
        self.prev_btn.clicked.connect(self.previous_question)
        nav_layout.addWidget(self.prev_btn)

        self.next_btn = QPushButton("Next")
        self.next_btn.clicked.connect(self.next_question)
        nav_layout.addWidget(self.next_btn)

        self.submit_btn = QPushButton("Submit Quiz")
        self.submit_btn.clicked.connect(self.submit_quiz)
        self.submit_btn.setStyleSheet("background-color: #4CAF50; color: white; font-weight: bold;")
        nav_layout.addWidget(self.submit_btn)

        layout.addLayout(nav_layout)

        self.setLayout(layout)
        self.update_navigation()

    def load_question(self):
        if not self.quiz.questions:
            QMessageBox.warning(self, "Error", "This quiz has no questions!")
            self.reject()
            return

        question = self.quiz.questions[self.current_question_index]

        # Update progress
        self.progress_label.setText(f"Question {self.current_question_index + 1} of {len(self.quiz.questions)}")

        # Update question text
        self.question_label.setText(question.question_text)

        # Update options
        for i, option in enumerate(question.options):
            self.option_buttons[i].setText(f"{chr(65 + i)}) {option}")
            self.option_buttons[i].setChecked(False)

        # Load previous answer if exists
        if self.current_question_index < len(self.user_answers):
            prev_answer = self.user_answers[self.current_question_index]
            if prev_answer is not None:
                self.option_buttons[prev_answer].setChecked(True)

    def update_navigation(self):
        self.prev_btn.setEnabled(self.current_question_index > 0)

        if self.current_question_index == len(self.quiz.questions) - 1:
            self.next_btn.setText("Finish")
        else:
            self.next_btn.setText("Next")

    def save_current_answer(self):
        checked_button = self.option_group.checkedButton()
        if checked_button:
            answer = self.option_group.id(checked_button)
            if self.current_question_index >= len(self.user_answers):
                self.user_answers.append(answer)
            else:
                self.user_answers[self.current_question_index] = answer

    def previous_question(self):
        self.save_current_answer()
        self.current_question_index -= 1
        self.load_question()
        self.update_navigation()

    def next_question(self):
        self.save_current_answer()

        if self.current_question_index == len(self.quiz.questions) - 1:
            self.submit_quiz()
        else:
            self.current_question_index += 1
            self.load_question()
            self.update_navigation()

    def submit_quiz(self):
        self.save_current_answer()

        # Calculate score
        self.score = 0
        results = []

        for i, (question, user_answer) in enumerate(zip(self.quiz.questions, self.user_answers)):
            is_correct = user_answer == question.correct_index
            if is_correct:
                self.score += 1

            results.append({
                'question': question.question_text,
                'user_answer': user_answer,
                'correct_answer': question.correct_index,
                'is_correct': is_correct,
                'options': question.options
            })

        # Show results
        self.show_results(results)

    def show_results(self, results):
        result_dialog = QDialog(self)
        result_dialog.setWindowTitle("Quiz Results")
        result_dialog.setGeometry(150, 150, 600, 400)

        layout = QVBoxLayout()

        # Score
        score_label = QLabel(f"Score: {self.score}/{len(self.quiz.questions)} ({self.score/len(self.quiz.questions)*100:.1f}%)")
        score_label.setFont(QFont("Arial", 20, QFont.Bold))
        score_label.setStyleSheet("color: #2196F3;")
        layout.addWidget(score_label)

        # Results scroll area
        scroll = QScrollArea()
        scroll_widget = QWidget()
        scroll_layout = QVBoxLayout()

        for i, result in enumerate(results):
            result_frame = QFrame()
            result_frame.setFrameStyle(QFrame.Box)
            result_frame.setStyleSheet("margin: 5px; padding: 10px;")

            frame_layout = QVBoxLayout()

            # Question
            q_label = QLabel(f"Q{i+1}: {result['question']}")
            q_label.setWordWrap(True)
            q_label.setFont(QFont('Ariel', 16))
            frame_layout.addWidget(q_label)

            # User answer
            if result['user_answer'] is not None:
                user_answer_text = f"Your answer: {chr(65 + result['user_answer'])}) {result['options'][result['user_answer']]}"
                user_label = QLabel(user_answer_text)
                user_label.setStyleSheet("color: #FF5722;" if not result['is_correct'] else "color: #4CAF50;")
                user_label.setFont(QFont('Ariel', 16))
                frame_layout.addWidget(user_label)
            else:
                user_label = QLabel("Your answer: (No answer)")
                user_label.setStyleSheet("color: #FF5722;")
                frame_layout.addWidget(user_label)

            # Correct answer
            if not result['is_correct']:
                correct_answer_text = f"Correct answer: {chr(65 + result['correct_answer'])}) {result['options'][result['correct_answer']]}"
                correct_label = QLabel(correct_answer_text)
                correct_label.setStyleSheet("color: #4CAF50; font-weight: bold;")
                correct_label.setFont(QFont('Ariel', 16))
                frame_layout.addWidget(correct_label)

            result_frame.setLayout(frame_layout)
            scroll_layout.addWidget(result_frame)

        scroll_widget.setLayout(scroll_layout)
        scroll.setWidget(scroll_widget)
        layout.addWidget(scroll)

        # Close button
        close_btn = QPushButton("Close")
        close_btn.clicked.connect(result_dialog.accept)
        layout.addWidget(close_btn)

        result_dialog.setLayout(layout)
        result_dialog.exec_()

class QuizApp(QMainWindow):
    def __init__(self):
        super().__init__()
        self.quiz_manager = QuizManager()
        self.current_quiz_index = -1
        self.init_ui()
        self.load_data()

    def init_ui(self):
        self.setWindowTitle("Quiz Manager")
        self.setGeometry(100, 100, 900, 600)

        central_widget = QWidget()
        self.setCentralWidget(central_widget)

        main_layout = QHBoxLayout()

        # Left side - Quiz list
        left_widget = QWidget()
        left_layout = QVBoxLayout()

        left_layout.addWidget(QLabel("Quizzes:"))
        self.quiz_list = QListWidget()
        self.quiz_list.currentRowChanged.connect(self.on_quiz_selected)
        left_layout.addWidget(self.quiz_list)

        quiz_buttons_layout = QHBoxLayout()
        self.add_quiz_btn = QPushButton("Add Quiz")
        self.edit_quiz_btn = QPushButton("Edit Quiz")
        self.delete_quiz_btn = QPushButton("Delete Quiz")
        self.take_quiz_btn = QPushButton("Take Quiz")
        self.take_quiz_btn.setStyleSheet("background-color: #4CAF50; color: white; font-weight: bold;")

        self.add_quiz_btn.clicked.connect(self.add_quiz)
        self.edit_quiz_btn.clicked.connect(self.edit_quiz)
        self.delete_quiz_btn.clicked.connect(self.delete_quiz)
        self.take_quiz_btn.clicked.connect(self.take_quiz)

        quiz_buttons_layout.addWidget(self.add_quiz_btn)
        quiz_buttons_layout.addWidget(self.edit_quiz_btn)
        quiz_buttons_layout.addWidget(self.delete_quiz_btn)
        quiz_buttons_layout.addWidget(self.take_quiz_btn)
        left_layout.addLayout(quiz_buttons_layout)

        left_widget.setLayout(left_layout)

        # Right side - Questions for selected quiz
        right_widget = QWidget()
        right_layout = QVBoxLayout()

        right_layout.addWidget(QLabel("Questions:"))
        self.question_list = QListWidget()
        self.question_list.currentRowChanged.connect(self.on_question_selected)
        right_layout.addWidget(self.question_list)

        question_buttons_layout = QHBoxLayout()
        self.add_question_btn = QPushButton("Add Question")
        self.edit_question_btn = QPushButton("Edit Question")
        self.delete_question_btn = QPushButton("Delete Question")

        self.add_question_btn.clicked.connect(self.add_question)
        self.edit_question_btn.clicked.connect(self.edit_question)
        self.delete_question_btn.clicked.connect(self.delete_question)

        question_buttons_layout.addWidget(self.add_question_btn)
        question_buttons_layout.addWidget(self.edit_question_btn)
        question_buttons_layout.addWidget(self.delete_question_btn)
        right_layout.addLayout(question_buttons_layout)

        # Question preview
        self.preview_group = QGroupBox("Question Preview")
        preview_layout = QVBoxLayout()

        self.preview_question = QLabel()
        self.preview_question.setWordWrap(True)
        self.preview_question.setFont(QFont("Arial", 10, QFont.Bold))
        preview_layout.addWidget(self.preview_question)

        self.preview_options = []
        for i in range(4):
            label = QLabel()
            label.setWordWrap(True)
            self.preview_options.append(label)
            preview_layout.addWidget(label)

        self.preview_correct = QLabel()
        self.preview_correct.setStyleSheet("color: green; font-weight: bold;")
        preview_layout.addWidget(self.preview_correct)

        self.preview_group.setLayout(preview_layout)
        right_layout.addWidget(self.preview_group)

        right_widget.setLayout(right_layout)

        # Splitter for resizable panels
        splitter = QSplitter(Qt.Horizontal)
        splitter.addWidget(left_widget)
        splitter.addWidget(right_widget)
        splitter.setSizes([300, 600])

        main_layout.addWidget(splitter)
        central_widget.setLayout(main_layout)

        # Menu bar
        menubar = self.menuBar()
        file_menu = menubar.addMenu('File')

        open_action = file_menu.addAction('Open Quiz File')
        open_action.triggered.connect(self.open_quiz_file)

        save_action = file_menu.addAction('Save All')
        save_action.triggered.connect(self.save_data)

        exit_action = file_menu.addAction('Exit')
        exit_action.triggered.connect(self.close)

    def load_data(self):
        self.quiz_manager.load_all()
        self.refresh_quiz_list()

    def save_data(self):
        self.quiz_manager.save_all()
        QMessageBox.information(self, "Success", "All quizzes saved successfully!")

    def open_quiz_file(self):
        filename, _ = QFileDialog.getOpenFileName(
            self,
            "Open Quiz File",
            "",
            "Quiz Files (*.json);;All Files (*)"
        )

        if filename:
            try:
                quiz = Quiz.load_from_file(filename)
                self.quiz_manager.add_quiz(quiz)
                self.refresh_quiz_list()
                QMessageBox.information(self, "Success", f"Quiz '{quiz.title}' loaded successfully!")
            except Exception as e:
                QMessageBox.warning(self, "Error", f"Failed to load quiz file: {str(e)}")

    def take_quiz(self):
        if self.current_quiz_index >= 0:
            quiz = self.quiz_manager.get_quiz(self.current_quiz_index)
            if quiz and quiz.questions:
                dialog = QuizDisplayDialog(quiz, self)
                dialog.exec_()
            else:
                QMessageBox.warning(self, "Warning", "Selected quiz has no questions!")
        else:
            QMessageBox.warning(self, "Warning", "Please select a quiz first!")

    def refresh_quiz_list(self):
        self.quiz_list.clear()
        for quiz in self.quiz_manager.quizzes:
            item = QListWidgetItem(f"{quiz.title} ({len(quiz.questions)} questions)")
            self.quiz_list.addItem(item)

    def refresh_question_list(self):
        self.question_list.clear()
        if self.current_quiz_index >= 0:
            quiz = self.quiz_manager.get_quiz(self.current_quiz_index)
            if quiz:
                for i, question in enumerate(quiz.questions):
                    # Truncate long question text for display
                    display_text = question.question_text
                    if len(display_text) > 50:
                        display_text = display_text[:50] + "..."
                    item = QListWidgetItem(f"Q{i+1}: {display_text}")
                    self.question_list.addItem(item)

    def on_quiz_selected(self, index):
        self.current_quiz_index = index
        self.refresh_question_list()
        self.preview_group.setVisible(False)

    def on_question_selected(self, index):
        if self.current_quiz_index >= 0 and index >= 0:
            quiz = self.quiz_manager.get_quiz(self.current_quiz_index)
            if quiz and index < len(quiz.questions):
                question = quiz.questions[index]
                self.show_question_preview(question)

    def show_question_preview(self, question):
        self.preview_question.setText(f"Q: {question.question_text}")

        for i, option in enumerate(question.options):
            prefix = "✓" if i == question.correct_index else "○"
            self.preview_options[i].setText(f"{prefix} {option}")

        self.preview_correct.setText(f"Correct answer: Option {question.correct_index + 1}")
        self.preview_group.setVisible(True)

    def add_quiz(self):
        title, ok = QInputDialog.getText(self, "New Quiz", "Enter quiz title:")
        if ok and title:
            quiz = Quiz(title=title)
            self.quiz_manager.add_quiz(quiz)
            self.refresh_quiz_list()
            self.quiz_list.setCurrentRow(len(self.quiz_manager.quizzes) - 1)

    def edit_quiz(self):
        if self.current_quiz_index >= 0:
            quiz = self.quiz_manager.get_quiz(self.current_quiz_index)
            if quiz:
                title, ok = QInputDialog.getText(self, "Edit Quiz", "Enter quiz title:", text=quiz.title)
                if ok and title:
                    quiz.title = title
                    self.refresh_quiz_list()

    def delete_quiz(self):
        if self.current_quiz_index >= 0:
            reply = QMessageBox.question(self, "Confirm Delete",
                                       "Are you sure you want to delete this quiz?",
                                       QMessageBox.Yes | QMessageBox.No)
            if reply == QMessageBox.Yes:
                self.quiz_manager.remove_quiz(self.current_quiz_index)
                self.current_quiz_index = -1
                self.refresh_quiz_list()
                self.refresh_question_list()

    def add_question(self):
        if self.current_quiz_index >= 0:
            dialog = QuestionEditorDialog()
            if dialog.exec_() == QDialog.Accepted:
                question = dialog.get_question()
                quiz = self.quiz_manager.get_quiz(self.current_quiz_index)
                quiz.add_question(question)
                self.refresh_question_list()
        else:
            QMessageBox.warning(self, "Warning", "Please select a quiz first!")

    def edit_question(self):
        if self.current_quiz_index >= 0 and self.question_list.currentRow() >= 0:
            quiz = self.quiz_manager.get_quiz(self.current_quiz_index)
            question_index = self.question_list.currentRow()
            question = quiz.questions[question_index]

            dialog = QuestionEditorDialog(question)
            if dialog.exec_() == QDialog.Accepted:
                updated_question = dialog.get_question()
                quiz.questions[question_index] = updated_question
                self.refresh_question_list()
                self.question_list.setCurrentRow(question_index)
        else:
            QMessageBox.warning(self, "Warning", "Please select a question first!")

    def delete_question(self):
        if (self.current_quiz_index >= 0 and
            self.question_list.currentRow() >= 0):

            reply = QMessageBox.question(self, "Confirm Delete",
                                       "Are you sure you want to delete this question?",
                                       QMessageBox.Yes | QMessageBox.No)
            if reply == QMessageBox.Yes:
                quiz = self.quiz_manager.get_quiz(self.current_quiz_index)
                quiz.remove_question(self.question_list.currentRow())
                self.refresh_question_list()
        else:
            QMessageBox.warning(self, "Warning", "Please select a question first!")

def main():
    app = QApplication(sys.argv)
    window = QuizApp()
    window.show()
    sys.exit(app.exec_())

if __name__ == '__main__':
    main()
