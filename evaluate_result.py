import os
import shutil
import argparse


def copy_files(problem_instance):
    """
    Copies files from training and result folders to evaluator folders.

    Args:
      problem_instance: The name of the folder containing the data.
    """
    # Define source and destination folders
    train_source = f"./Project 3 training_images/{problem_instance}"
    eval_train_dest = f"./evaluator/optimal_segments"
    result_source = f"./logs/result_segmentation/{problem_instance}"
    eval_result_dest = f"./evaluator/student_segments"

    # Create evaluator folders if they don't exist
    os.makedirs(eval_train_dest, exist_ok=True)
    os.makedirs(eval_result_dest, exist_ok=True)

    # Clear existing files in evaluator folders
    for filename in os.listdir(eval_train_dest):
        file_path = os.path.join(eval_train_dest, filename)
        os.remove(file_path)
    for filename in os.listdir(eval_result_dest):
        file_path = os.path.join(eval_result_dest, filename)
        os.remove(file_path)

    # Copy files with "GT" prefix from training folder
    for filename in os.listdir(train_source):
        if filename.startswith("GT"):
            source_path = os.path.join(train_source, filename)
            dest_path = os.path.join(eval_train_dest, filename)
            shutil.copy2(source_path, dest_path)  # Preserves file metadata

    # Copy all files from result folder
    for filename in os.listdir(result_source):
        source_path = os.path.join(result_source, filename)
        dest_path = os.path.join(eval_result_dest, filename)
        shutil.copy2(source_path, dest_path)


if __name__ == "__main__":
    # Parse command-line arguments
    parser = argparse.ArgumentParser(description="Copy files for evaluation.")
    parser.add_argument("problem_instance", type=str,
                        help="Name of the problem instance folder.")
    args = parser.parse_args()

    # Call copy_files function with the parsed argument
    copy_files(args.problem_instance)

    os.system("python ./evaluator/run.py")
