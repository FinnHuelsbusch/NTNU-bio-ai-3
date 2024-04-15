import pandas as pd
import os
import numpy as np
import matplotlib.pyplot as plt

from nds import ndomsort


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
        df = pd.DataFrame(content_dict)

        
        df['edge_value_fitness'] = -df['edge_value_fitness']
        fronts = ndomsort.non_domin_sort(np.array(df[['connectivity_fitness', 'overall_deviation_fitness', 'edge_value_fitness']]))
        for key, value in fronts.items():
            for entry in value:
                connectivity_fitness = entry[0]
                overall_deviation_fitness = entry[1]
                edge_value_fitness = entry[2]
                df.loc[(df['connectivity_fitness'] == connectivity_fitness) & (df['overall_deviation_fitness'] == overall_deviation_fitness) & (df['edge_value_fitness'] == edge_value_fitness), 'python_front'] = key

        # check if fronts are equal
        if not np.array_equal(df['front'], df['python_front']):
            print(f'Fronts are not equal for file {file}')
            print(df)
        # print(df)

        
