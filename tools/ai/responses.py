import numpy as np
import matplotlib.pyplot as plt

def func1(x):
    fx = np.arange(0.1, x, 0.5)
    fy = np.log(fx)
    plt.plot(fx, fy)
    plt.show()
    return x