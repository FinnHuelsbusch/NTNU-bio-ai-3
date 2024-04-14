import seaborn as sns 
import matplotlib.pyplot as plt
import pandas as pd
import os
from mpl_toolkits.mplot3d import Axes3D

dataframes = {}
for file in [f for f in os.listdir('./logs') if f.endswith('.txt')]:
    with open(f'./logs/{file}', 'r') as f:
        content_dict = {
            "front": [],
            "connectivity_fitness": [],
            "overall_deviation_fitness": [],
            "edge_value_fitness": []
        }
        data = f.readlines()
        for index, line in enumerate(data):
            line = line.replace('\n', '')
            individuals = line.split(';')
            for individual in individuals:
                if individual == '':
                    continue
                individual = individual.replace('(', '').replace(')', '').split(',')
                content_dict["front"].append(index)
                content_dict["edge_value_fitness"].append(float(individual[0]))
                content_dict["connectivity_fitness"].append(float(individual[1]))
                content_dict["overall_deviation_fitness"].append(float(individual[2]))
        dataframes[file.split(".")[0].split("_")[-1]] = pd.DataFrame(content_dict)

def export_all(): 
    for key, df in dataframes.items():
        fig = plt.figure()
        ax = fig.add_subplot(projection='3d')
        for front in df['front'].unique():
            front_df = df[df['front'] == front]
            ax.scatter(front_df['edge_value_fitness'], front_df['connectivity_fitness'], front_df['overall_deviation_fitness'], label=f'Front {front}')
        ax.set_xlabel('Edge Value Fitness (max)')
        ax.set_ylabel('Connectivity Fitness (min)')
        ax.set_zlabel('Overall Deviation Fitness (min)')
        ax.set_title('Pareto Front visualization')
        plt.legend()




        plt.savefig(f'./logs/{key}.png')
        plt.close()

def visualize_interactive(index):
    df = dataframes[str(index)]
    fig = plt.figure()
    ax = fig.add_subplot(projection='3d')
    for front in df['front'].unique():
        front_df = df[df['front'] == front]
        ax.scatter(front_df['edge_value_fitness'], front_df['connectivity_fitness'], front_df['overall_deviation_fitness'], label=f'Front {front}')
    ax.set_xlabel('Edge Value Fitness (max)')
    ax.set_ylabel('Connectivity Fitness (min)')
    ax.set_zlabel('Overall Deviation Fitness (min)')
    ax.set_title('Pareto Front visualization')
    plt.legend()
    plt.show()

exit = False
while not exit:
    print('1. Export all Pareto Fronts\n2. Visualize Pareto Fronts\n3. Exit')
    option = input('Choose an option: ')
    if option == '1':
        export_all()
    elif option == '2':
        index = int(input('Enter the index of the pareto front: '))
        visualize_interactive(index)
    elif option == '3':
        exit = True
    else:
        print('Invalid option')
