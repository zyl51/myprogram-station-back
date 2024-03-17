# 1.将式子等价变换
$x ≡ m (mod ,a)  -->  x = k * a + m$
### 所以有:

$x = k_1 * a_1 + m_1$
$x = k_2 * a_2 + m_2$
$k_1 * a_1 + m_1 = k_2 * a_2 + m_2$
$k_1 * a_1 - k_2 * a_2 = m_2 - m_1$

令$gcd(a_1, a_2) = gcd$
所以只有$(m_2 - m_1) \% gcd == 0 $才有解 
现在，我们就是求一下这个式子$ k_1 * a_1 - k_2 * a_2 = gcd$
只要把求出的$ k_1, k_2 $扩大相应的倍数$(m_2 - m_1) % gcd $就可以了

## 扩展欧几里得得出$k_1, k_2$

$ gcd(a, b) == gcd(b, a \% b)$
$ a * x + b * y == gcd(a, b) $
--> $b * x_1 + (a \% b) * y_1 == gcd(b, a \% b) $
--> $b * x_1 + (a - a / b * b) * y_1 == gcd(b, a \% b) $
--> $a * y_1 + b * (x_1 - a / b * y_1) == gcd(b, a \% b) $
当$ b == 0 $的情况下，显然有一个解！！！ $x_1 = 1，y1 = 0$
所以我们只要递归求此过程
$x = y_1 , y = x_1 - a / b * y_1$ 

### 得到$k_1$和$k_2$之后 

对于式子：$k_1 * a_1 - k_2 * a_2 = gcd$
其中，我们可以想到系数 $k$ 可以进行变换
$k1 = k1 + k * (a2 / gcd)$
$k2 = k2 + k * (a1 / gcd)$

## 证明：

$(k_1 + k * (a_2 / gcd)) * a_1 - (k_2 + k * (a_1 / gcd)) * a2 == gcd$
展开之后 我们发现 $k * (a_2 / gcd) * a_1 == k * (a_1 / gcd) * a_2$
所以变形等价

### 最后将k变换缩小

变换：$k_1 = k_1 * (m_2 - m_1) / gcd$
我们令：$d = abs(a_2 / gcd)$
缩小：$k_1 = (k_1 \% d + d ) \% d$

### 将 $k_1 + k * (a_2 / gcd)$ 带入式子中

原式：$x = k_1 * a_1 + m_1$
变换：$x = (k_1 + k * (a_2 / gcd)) * a_1 + m_1$
令$m_0 = k_1 * a_1 + m_1$
令$a_0 = a_2 / gcd *a_1 = lcm(a_1, a_2)$

# 最后得出结果！！！！

$ x = k * a_0 + m_0$
# 可以看出与之前的类似，所以我们只要重复这个操作$ n - 1 $次就可以了！！
```
#include<iostream>
using namespace std;
typedef long long ll;

int read()
{
    int x = 0, y = 1; char c = getchar();
    while(c < '0' || c > '9'){if(c == '-') y = - 1; c = getchar(); }
    while(c >='0' && c <= '9'){x = (x << 1) + (x << 3) + (c ^ 48); c = getchar();};
    return x * y;
}

ll exgcd(ll a, ll b, ll &x, ll &y)
{
    if(b == 0)
    {
        x = 1, y = 0;
        return a;
    }
    ll gcd, x1, y1;
    gcd = exgcd(b, a % b, x1, y1);
    x = y1;
    y = x1 - a / b * y1;
    return gcd;
}

ll mod(ll a,ll b)
{
    return (a % b + b) % b; 
}

int main()
{
    int n = read();
    ll a1 = read(), m1 = read();
    bool flag = true;
    for(int i = 1;i <= n - 1; i ++ )
    {
        ll k1, k2;
        ll a2 = read(), m2 = read();
        ll gcd = exgcd(a1, -a2, k1, k2);
        if((m2 - m1) % gcd == 0)
        {
            k1 = mod(k1 * (m2 - m1) / gcd,abs(a2 / gcd));
            m1 = k1 * a1 + m1;
            a1 = abs(a1 * a2 / gcd);
        }
        else flag = false;
    }
    if(flag)
    {
        cout << m1 <<endl;
    }
    else puts("-1");
    return 0;
}
```