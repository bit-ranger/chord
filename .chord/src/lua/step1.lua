r = os.time();
t =  "CHORD-" .. tostring(r);
print(t);
return
{
    {
        ['foo'] = var.foo
    }
,
    {
        ['bar'] = tonumber(var.bar)
    },
    {
        ['tag'] = t
    }
}