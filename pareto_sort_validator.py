import pandas as pd
import os
import numpy as np
import matplotlib.pyplot as plt

def is_pareto_efficient_dumb(costs):
    """
    Find the pareto-efficient points
    :param costs: An (n_points, n_costs) array
    :return: A (n_points, ) boolean array, indicating whether each point is Pareto efficient
    """
    is_efficient = np.ones(costs.shape[0], dtype = bool)
    for i, c in enumerate(costs):
        is_efficient[i] = np.all(np.any(costs[:i]>c, axis=1)) and np.all(np.any(costs[i+1:]>c, axis=1))
    return is_efficient


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
        pareto_mask = is_pareto_efficient_dumb(df[['connectivity_fitness', 'overall_deviation_fitness', 'edge_value_fitness']].values)
        df['Pareto'] = 'No'
        df.loc[pareto_mask, 'Pareto'] = 'Yes'
        #if any pair exists where Pareto yes and front is 1, or Pareto no and front is 0
        if df['front'].nunique() == 1 and df['front'].unique()[0] == 0:
            continue

        if any((df['Pareto'] == 'Yes') & (df['front'] != 0)) or any((df['Pareto'] == 'No') & (df['front'] == 0)):
            print(file)
            df['edge_value_fitness'] = -df['edge_value_fitness']
            df.sort_values(['connectivity_fitness', 'overall_deviation_fitness', 'edge_value_fitness'], inplace=True)
            print(df)

        
