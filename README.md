# Bio-Inspired Artificial Intelligence - Handin 3
This repository contains the third handin for the course Bio-Inspired Artificial Intelligence IT3708 at NTNU.

## The task
The task is to do multi-objective optimization for image segmentaiton problems. The three objectives are:
- maximize the edge value: $\begin{aligned} & \text { Edge value }:=\sum_{i=1}^N\left(\sum_{j \in F_i} x_{i, j}\right) \\ & \text { where } x_{i, j}=\left\{\begin{array}{cl}\operatorname{dist}(i, j) & \text { if } \nexists C_k: i, j \in C_k \\ 0 & \text { otherwise }\end{array}\right. \\ & \end{aligned}$
- minimize the connectivity measure: $\begin{aligned} & \text { Connectivity }:=\sum_{i=1}^N\left(\sum_{j \in F_i} x_{i, j}\right), \\ & \text { where } x_{i, j}=\left\{\begin{array}{cl}\frac{1}{F_i(j)} & \text { if } \nexists C_k: i, j \in C_k \\ 0 & \text { otherwise }\end{array}\right.\end{aligned}$
- minimize overall deviation from the average of a cluster: $
\text { Overall-deviation }:=\sum_{C_k \in C} \sum_{i \in C_k} \operatorname{dist}\left(i, \mu_k\right)
$

where $N$ is the number of pixels, $F$ is the set of neighboring pixels, $C$ is the set of clusters, $C$ is a cluster. 