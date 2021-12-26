r = os.time();
t =  "CHORD-" .. tostring(r);
print(t);
return
{
    {
        ['foo'] = foo
    }
,
    {
        ['bar'] = tonumber(bar)
    },
    {
        ['tag'] = t
    }
}