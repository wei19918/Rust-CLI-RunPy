from datetime import datetime
import random
import string

r = ''.join(random.choices(string.ascii_letters + string.digits, k=8))
print(f"running a random job--{r}")
print(f"job at {datetime.now().strftime('%Y-%m-d %H:%M:%S')}")
