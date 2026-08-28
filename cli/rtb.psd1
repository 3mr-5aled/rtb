@{
    RootModule        = 'rtb.psm1'
    ModuleVersion     = '1.0.0'
    GUID              = 'a1b2c3d4-e5f6-7890-abcd-ef1234567890'
    Author            = 'devamr'
    Description       = 'RTB — Repository & Tooling Base (rtb) CLI'
    PowerShellVersion = '7.0'
    FunctionsToExport = @('rtb', 'Get-AllProjectNames', 'Get-ProjectsByStatus', 'Find-ProjectPath', 'Get-RtbConfig')
    CmdletsToExport   = @()
    VariablesToExport  = @()
    AliasesToExport   = @()
}
